use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Color32},
    EguiContexts,
};

use super::AppState;

pub struct StartMenuPlugin;

impl Plugin for StartMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(start_menu_setup.in_schedule(OnEnter(AppState::StartMenu)))
            .add_system(start_menu_system.in_set(OnUpdate(AppState::StartMenu)))
            .add_system(start_menu_cleanup.in_schedule(OnExit(AppState::StartMenu)));
    }
}

pub fn start_menu_setup() {
    println!("Start Menu setup");
}

pub fn start_menu_system(mut contexts: EguiContexts, mut app_state: ResMut<NextState<AppState>>) {
    let context = contexts.ctx_mut();
    let mut visuals = context.style().visuals.clone();
    visuals.window_fill = Color32::BLUE;
    visuals.window_stroke.width = 0.0;
    context.set_visuals(visuals);

    egui::Window::new("Start Menu").show(context, |ui| {
        if ui.button("Start Game").clicked() {
            // Start game
            app_state.set(AppState::DeviceSelect);
        }

        if ui.button("Settings").clicked() {
            // Settings
        }
    });
}

pub fn start_menu_cleanup() {
    println!("Start Menu cleanup");
}
