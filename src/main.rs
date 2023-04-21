use bevy::{prelude::*, window::WindowResolution};
use bevy_egui::EguiPlugin;
use bevy_rapier3d::prelude::*;

use debug::DebugPlugin;
use midi::MidiInputPlugin;
use states::AppStatePlugin;

mod debug;
mod midi;
mod states;

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
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(EguiPlugin)
        .add_plugin(MidiInputPlugin)
        .add_plugin(AppStatePlugin)
        .add_plugin(DebugPlugin)
        .run();
}
