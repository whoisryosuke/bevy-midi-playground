use bevy::{prelude::*, window::WindowResolution};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use chrono::{TimeZone, Utc};

use crossbeam_channel::{Receiver, Sender};
use midi::{MidiInputPlugin, MidiSetupState, SelectDeviceEvent};

mod midi;

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
        .add_plugin(MidiInputPlugin)
        .add_system(select_device_ui)
        .run();
}

// The UI for selecting a device
fn select_device_ui(
    mut contexts: EguiContexts,
    midi_state: Res<MidiSetupState>,
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
