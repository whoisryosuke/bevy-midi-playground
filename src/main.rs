use bevy::{prelude::*, window::WindowResolution};
use bevy_egui::{egui, EguiContexts, EguiPlugin};

use midir::{Ignore, MidiInput, MidiInputPort};

// App state to store and manage notifications
#[derive(Resource)]
pub struct MidiState {
    // An instance to access MIDI devices and input
    input: MidiInput,
    // Available ports
    available_ports: Vec<MidiInputPort>,
    // The ID of currently selected device's port
    selected_port: Option<usize>,
}

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
        .add_startup_system(setup_midi)
        .add_system(hello_world)
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

fn select_device(mut midi_state: ResMut<MidiState>) {
    // Is there a device selected? Skip this system then.
    if midi_state.selected_port.is_some() {
        return;
    }

    // Get all available ports
    midi_state.available_ports = midi_state.input.ports();
}

fn select_device_ui(mut contexts: EguiContexts, mut midi_state: ResMut<MidiState>) {
    let context = contexts.ctx_mut();
    egui::Window::new("Select a MIDI device").show(context, |ui| {
        let ports = midi_state.available_ports.iter().enumerate();
        for (index, port) in ports {
            if ui
                .button(midi_state.input.port_name(port).unwrap())
                .clicked()
            {
                // midi_state.selected_port = Some(index);
            }
        }
    });
}

fn hello_world() {
    // println!("Testing");
}
