use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Color32},
    EguiContexts,
};

use super::AppState;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(game_setup.in_schedule(OnEnter(AppState::Game)))
            .add_system(game_system.in_set(OnUpdate(AppState::Game)))
            .add_system(game_cleanup.in_schedule(OnExit(AppState::Game)));
    }
}

pub fn game_setup() {
    println!("Game setup");
}

pub fn game_system() {}

pub fn game_cleanup() {
    println!("Game cleanup");
}
