use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

// App-level debug state
#[derive(Resource)]
pub struct DebugState {
    // Is debug menu visible?
    pub visible: bool,
    // A general position value to play with
    pub debug_position: Vec3,
    pub camera_look: Vec3,
    pub rotation: Vec3,
}

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DebugState {
            visible: false,
            debug_position: Vec3::splat(0.0),
            camera_look: Vec3::new(0.0, 10.0, 0.0),
            rotation: Vec3::splat(0.0),
        })
        .add_system(debug_ui)
        .add_system(debug_controls);
    }
}

fn debug_ui(mut contexts: EguiContexts, mut debug_state: ResMut<DebugState>) {
    if debug_state.visible {
        egui::Window::new("Debug").show(contexts.ctx_mut(), |ui| {
            ui.heading("General");
            ui.horizontal(|ui| {
                ui.label("Position");
                ui.add(egui::DragValue::new(&mut debug_state.debug_position.x).speed(0.1));
                ui.add(egui::DragValue::new(&mut debug_state.debug_position.y).speed(0.1));
                ui.add(egui::DragValue::new(&mut debug_state.debug_position.z).speed(0.1));
            });
            ui.horizontal(|ui| {
                ui.label("Camera target");
                ui.add(egui::DragValue::new(&mut debug_state.camera_look.x).speed(0.1));
                ui.add(egui::DragValue::new(&mut debug_state.camera_look.y).speed(0.1));
                ui.add(egui::DragValue::new(&mut debug_state.camera_look.z).speed(0.1));
            });
            ui.horizontal(|ui| {
                ui.label("Rotation");
                ui.add(egui::DragValue::new(&mut debug_state.rotation.x).speed(0.1));
                ui.add(egui::DragValue::new(&mut debug_state.rotation.y).speed(0.1));
                ui.add(egui::DragValue::new(&mut debug_state.rotation.z).speed(0.1));
            });
        });
    }
}

fn debug_controls(keyboard_input: Res<Input<KeyCode>>, mut debug_state: ResMut<DebugState>) {
    if keyboard_input.pressed(KeyCode::LShift) && keyboard_input.just_released(KeyCode::P) {
        if debug_state.visible {
            debug_state.visible = false;
        } else {
            debug_state.visible = true;
        }
    }
}
