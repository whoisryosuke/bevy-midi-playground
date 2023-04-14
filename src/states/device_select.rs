use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Color32},
    EguiContexts,
};

use crate::midi::{MidiInputState, MidiSetupState, SelectDeviceEvent};

use super::AppState;

pub struct DeviceSelectPlugin;

impl Plugin for DeviceSelectPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(device_select_setup.in_schedule(OnEnter(AppState::DeviceSelect)))
            .add_system(device_select_ui.in_set(OnUpdate(AppState::DeviceSelect)))
            .add_system(device_select_redirect.in_set(OnUpdate(AppState::DeviceSelect)))
            .add_system(device_select_cleanup.in_schedule(OnExit(AppState::DeviceSelect)));
    }
}

pub fn device_select_setup() {
    println!("Device Select setup");
}

pub fn device_select_cleanup() {
    println!("Device Select cleanup");
}

// Once we've connected, we should redirect to game
pub fn device_select_redirect(
    input_state: Res<MidiInputState>,
    mut app_state: ResMut<NextState<AppState>>,
) {
    if input_state.connected {
        println!("redirecting to game");
        app_state.set(AppState::Game);
    }
}

// The UI for selecting a device
fn device_select_ui(
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
