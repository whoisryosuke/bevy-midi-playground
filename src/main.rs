use bevy::{ecs::system::SystemState, prelude::*, window::WindowResolution};
use bevy_egui::{egui, EguiContexts, EguiPlugin};

use midir::{Ignore, MidiInput, MidiInputConnection, MidiInputPort};

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
        .add_system(hello_world)
        .add_system(discover_devices)
        .add_system(select_device)
        .add_system(select_device_ui)
        .run();
}

fn setup_midi(mut commands: Commands) {
    let mut midi_in = MidiInput::new("midir reading input").expect("Couldn't initialize MidiInput");
    midi_in.ignore(Ignore::None);

    commands.insert_resource(MidiState {
        input: midi_in,
        available_ports: Vec::new(),
        selected_port: None,
    });
}

fn discover_devices(mut midi_state: ResMut<MidiState>) {
    // Is there a device selected? Skip this system then.
    if midi_state.selected_port.is_some() {
        return;
    }

    // Get all available ports
    midi_state.available_ports = midi_state.input.ports();
}

fn select_device(world: &mut World) {
    let mut event_system_state = SystemState::<(EventReader<SelectDeviceEvent>)>::new(world);
    let (mut device_events) = event_system_state.get(&world);

    let mut connection_result = None;

    if !device_events.is_empty() {
        for device_event in device_events.iter() {
            // Get the port from the event
            let SelectDeviceEvent(device_id) = device_event;

            let mut input =
                MidiInput::new("midir reading input").expect("Couldn't initialize MidiInput");
            input.ignore(Ignore::None);
            let ports = input.ports();

            match ports.get(*device_id).ok_or("invalid input port selected") {
                Ok(device_port) => {
                    println!("Connecting...");
                    let _conn_in = input
                        .connect(
                            device_port,
                            "midir-read-input",
                            move |stamp, message, _| {
                                println!("{}: {:?} (len = {})", stamp, message, message.len());

                                // stamp = incrementing time
                                // message = array of keyboard data. [keyEvent, keyId, strength]
                                // key
                            },
                            (),
                        )
                        .expect("Couldn't connect to that port. Did the devices change recently?");
                    connection_result = Some(_conn_in);
                }
                Err(error) => {
                    println!("Error {}", error);
                }
            }
            // let in_port_name = midi_state.input.port_name(device_id).expect("Couldn't connect to that port. Did the devices change recently?");
        }

        if let Some(connection) = connection_result {
            world.insert_non_send_resource(connection);
        }

        // device_events.clear();
    }
}

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

fn hello_world() {
    // println!("Testing");
}
