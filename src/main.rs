use bevy::{prelude::*, window::WindowResolution};
use bevy_egui::EguiPlugin;

use midi::MidiInputPlugin;
use states::AppStatePlugin;

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
        .add_plugin(EguiPlugin)
        .add_plugin(MidiInputPlugin)
        .add_plugin(AppStatePlugin)
        .run();
}
