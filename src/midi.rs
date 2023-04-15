use core::fmt;

use bevy::{ecs::system::SystemState, prelude::*};
use bevy_egui::{egui, EguiContexts};
use crossbeam_channel::{Receiver, Sender};
use midir::{MidiInput, MidiInputPort};

use crate::states::AppState;

// Structs
const KEY_HISTORY_LENGTH: usize = 10;

// State to manage
#[derive(Resource)]
pub struct MidiSetupState {
    // An instance to access MIDI devices and input
    pub input: MidiInput,
    // Available ports
    pub available_ports: Vec<MidiInputPort>,
    // The ID of currently selected device's port
    pub selected_port: Option<MidiInputPort>,
}

pub enum MidiResponse {
    Input(MidiInputKey),
    Connected,
    Disconnected,
    // Error(String),
}

#[derive(Resource)]
pub struct MidiInputReader {
    receiver: Receiver<MidiResponse>,
    sender: Sender<MidiResponse>,
}

#[derive(Resource)]
pub struct MidiInputState {
    // Is a device connected?
    pub connected: bool,
    // History of last pressed keys
    pub keys: Vec<MidiInputKey>,
    // Octave offset
    pub octave: i32,
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum MidiEvents {
    #[default]
    Pressed,
    Released,
    Holding,
}

impl fmt::Display for MidiEvents {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MidiEvents::Pressed => write!(f, "Pressed"),
            MidiEvents::Released => write!(f, "Released"),
            MidiEvents::Holding => write!(f, "Holding"),
        }
    }
}

// Event for MIDI key input
#[derive(Default, Clone, Copy)]
pub struct MidiInputKey {
    pub timestamp: u64,
    pub event: MidiEvents,
    pub id: u8,
    pub intensity: u8,
}

// Event to trigger a notification
#[derive(Default)]
pub struct SelectDeviceEvent(pub usize);

// Plugin
pub struct MidiInputPlugin;

impl Plugin for MidiInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SelectDeviceEvent>()
            .add_event::<MidiInputKey>()
            .insert_resource(MidiInputState {
                connected: false,
                keys: Vec::new(),
                octave: 0,
            })
            .add_startup_system(setup_midi)
            .add_system(discover_devices)
            .add_system(sync_keys)
            .add_system(select_device)
            .add_system(debug_input_ui);
    }
}

// Initializes the MIDI input instance and adds as a resource
fn setup_midi(mut commands: Commands) {
    let mut midi_in = MidiInput::new("midir reading input").expect("Couldn't initialize MidiInput");
    midi_in.ignore(midir::Ignore::None);

    commands.insert_resource(MidiSetupState {
        input: midi_in,
        available_ports: Vec::new(),
        selected_port: None,
    });

    // We create a message channel to communicate between MIDI protocol and Bevy state
    let (sender, receiver) = crossbeam_channel::unbounded::<MidiResponse>();
    commands.insert_resource(MidiInputReader {
        sender: sender,
        receiver: receiver,
    });
}

// Constantly updates available devices
fn discover_devices(mut midi_state: ResMut<MidiSetupState>) {
    // Is there a device selected? Skip this system then.
    if midi_state.selected_port.is_some() {
        return;
    }

    // Get all available ports
    midi_state.available_ports = midi_state.input.ports();
}

// Checks MIDI message channel and syncs changes with Bevy (like input or connectivity)
fn sync_keys(
    input_reader: Res<MidiInputReader>,
    mut input_state: ResMut<MidiInputState>,
    mut key_events: EventWriter<MidiInputKey>,
) {
    if let Ok(message) = input_reader.receiver.try_recv() {
        match message {
            MidiResponse::Input(input) => {
                println!("Key detected: {}", input.id);

                // Send event with latest key input
                key_events.send(input.clone());

                // Clear previous key history if it exceeds max size
                while input_state.keys.len() >= KEY_HISTORY_LENGTH {
                    input_state.keys.remove(0);
                }
                input_state.keys.push(input.clone());
            }
            MidiResponse::Connected => {
                input_state.connected = true;
            }
            MidiResponse::Disconnected => {
                input_state.connected = false;
            }
        }
    }
}

// Checks for device connection events, connects to device, and stores connection as resource
fn select_device(world: &mut World) {
    // Query the events using the world
    // We do this here since any system using World can't have other parameters
    let mut event_system_state =
        SystemState::<(EventReader<SelectDeviceEvent>, Res<MidiInputReader>)>::new(world);
    let (mut device_events, input_reader) = event_system_state.get(&world);

    // Store the connection in an optional variable
    let mut connection_result = None;

    // Loop over all device events if there's any
    if !device_events.is_empty() {
        for device_event in device_events.iter() {
            // Get the port from the event
            let SelectDeviceEvent(device_id) = device_event;

            // Create a new MIDI input instance
            // We do this here instead of using MidiSetupState because `connect()` consumes instance
            let mut input =
                MidiInput::new("midir reading input").expect("Couldn't initialize MidiInput");
            input.ignore(midir::Ignore::None);
            let ports = input.ports();
            let sender = input_reader.sender.clone();

            // Grab the port based on the port index from the event
            match ports.get(*device_id).ok_or("invalid input port selected") {
                Ok(device_port) => {
                    println!("Connecting to... {}", device_id);
                    // Connect to device!
                    let _conn_in = input.connect(
                        device_port,
                        "midir-read-input",
                        move |stamp, message, _| {
                            // println!("{}: {:?} (len = {})", stamp, message, message.len());
                            // stamp = incrementing time
                            // message = array of keyboard data. [keyEvent, keyId, strength]

                            // @TODO: Figure out system for determining input for different array sizes
                            if message.len() < 3 {
                                return;
                            }

                            let event_type = match message[0] {
                                144 => MidiEvents::Pressed,
                                128 => MidiEvents::Released,
                                160 => MidiEvents::Holding,
                                _ => MidiEvents::Pressed,
                            };

                            // Send the key via message channel to reach outside this callback
                            sender.send(MidiResponse::Input(MidiInputKey {
                                timestamp: stamp,
                                event: event_type,
                                id: message[1],
                                intensity: message[2],
                            }));
                        },
                        (),
                    );

                    match _conn_in {
                        Ok(connection) => {
                            input_reader.sender.send(MidiResponse::Connected);

                            // Store the connection for later
                            connection_result = Some(connection);
                        }
                        Err(_) => {}
                    }
                }
                Err(error) => {
                    println!("Error {}", error);
                }
            }
        }

        // Add the connection as a "non-send" resource.
        // Lets it persist past this system.
        // And connection can't be used across threads so this enforces main thread only
        if let Some(connection) = connection_result {
            world.insert_non_send_resource(connection);
        }
    }
}

// The UI for selecting a device
fn debug_input_ui(
    mut contexts: EguiContexts,
    input_state: Res<MidiInputState>,
    app_state: Res<State<AppState>>,
) {
    // Only display during game
    if app_state.0 != AppState::Game {
        return;
    }

    let context = contexts.ctx_mut();
    egui::Window::new("Input state").show(context, |ui| {
        // Connected status
        let mut name: String;
        if input_state.connected {
            name = "🟢 Connected".to_string();
        } else {
            name = "🔴 Disconnected".to_string();
        }
        ui.heading(name);

        ui.heading("Input history");
        for key in input_state.keys.iter() {
            ui.horizontal(|ui| {
                // let date_time = Utc.timestamp_millis_opt(key.timestamp as i64).unwrap();
                ui.horizontal(|ui| {
                    ui.strong("Time");
                    // ui.label(date_time.timestamp_millis().to_string());
                    ui.label(key.timestamp.to_string());
                });

                let name = key.id.to_string();
                ui.horizontal(|ui| {
                    ui.strong("Key");
                    ui.label(name);
                });

                let event = key.event.to_string();
                ui.horizontal(|ui| {
                    ui.strong("Event");
                    ui.label(event);
                });

                let intensity = key.intensity.to_string();
                ui.horizontal(|ui| {
                    ui.strong("Intensity");
                    ui.label(intensity);
                });
            });
        }
    });
}
