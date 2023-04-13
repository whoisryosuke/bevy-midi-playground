use bevy::{ecs::system::SystemState, prelude::*, window::WindowResolution};
use bevy_egui::{egui, EguiContexts, EguiPlugin};

use crossbeam_channel::{Receiver, Sender};
use midir::{Ignore, MidiInput, MidiInputPort};

// App state to store and manage notifications
#[derive(Resource)]
pub struct MidiState {
    // An instance to access MIDI devices and input
    input: MidiInput,
    // Available ports
    available_ports: Vec<MidiInputPort>,
    // The ID of currently selected device's port
    selected_port: Option<MidiInputPort>,
}

pub struct MidiResponse {
    key: u8,
}

#[derive(Resource)]
pub struct MidiInputReader {
    receiver: Receiver<MidiResponse>,
    sender: Sender<MidiResponse>,
}

// Event to trigger a notification
#[derive(Default)]
struct SelectDeviceEvent(usize);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(1024., 768.),
                title: "Bevy MIDI Revolution".to_string(),
                ..default()
            }),
            ..default()
        }))
        .add_plugin(EguiPlugin)
        .add_event::<SelectDeviceEvent>()
        .add_startup_system(setup_midi)
        .add_system(discover_devices)
        .add_system(sync_keys)
        .add_system(select_device)
        .add_system(select_device_ui)
        .run();
}

// Initializes the MIDI input instance and adds as a resource
fn setup_midi(mut commands: Commands) {
    let mut midi_in = MidiInput::new("midir reading input").expect("Couldn't initialize MidiInput");
    midi_in.ignore(Ignore::None);

    commands.insert_resource(MidiState {
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
fn discover_devices(mut midi_state: ResMut<MidiState>) {
    // Is there a device selected? Skip this system then.
    if midi_state.selected_port.is_some() {
        return;
    }

    // Get all available ports
    midi_state.available_ports = midi_state.input.ports();
}

// Checks MIDI message channel for new key inputs each frame
fn sync_keys(input_reader: Res<MidiInputReader>) {
    if let Ok(message) = input_reader.receiver.try_recv() {
        println!("Key detected: {}", message.key);
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
            // We do this here instead of using MidiState because `connect()` consumes instance
            let mut input =
                MidiInput::new("midir reading input").expect("Couldn't initialize MidiInput");
            input.ignore(Ignore::None);
            let ports = input.ports();
            let sender = input_reader.sender.clone();

            // Grab the port based on the port index from the event
            match ports.get(*device_id).ok_or("invalid input port selected") {
                Ok(device_port) => {
                    println!("Connecting...");
                    // Connect to device!
                    let _conn_in = input
                        .connect(
                            device_port,
                            "midir-read-input",
                            move |stamp, message, _| {
                                println!("{}: {:?} (len = {})", stamp, message, message.len());
                                // stamp = incrementing time
                                // message = array of keyboard data. [keyEvent, keyId, strength]

                                // Send the key via message channel to reach outside this callback
                                sender.send(MidiResponse { key: message[1] });
                            },
                            (),
                        )
                        .expect("Couldn't connect to that port. Did the devices change recently?");

                    // Store the connection for later
                    connection_result = Some(_conn_in);
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
fn select_device_ui(
    mut contexts: EguiContexts,
    midi_state: Res<MidiState>,
    mut device_event: EventWriter<SelectDeviceEvent>,
) {
    let context = contexts.ctx_mut();
    egui::Window::new("Select a MIDI device").show(context, |ui| {
        let ports = midi_state.available_ports.iter().enumerate();
        for (index, port) in ports {
            let device_name = midi_state.input.port_name(port).unwrap();
            if ui.button(&device_name).clicked() {
                // midi_state.selected_port = Some(index);
                println!("Selecting device {}", &device_name);
                device_event.send(SelectDeviceEvent(index));
            }
        }
    });
}
