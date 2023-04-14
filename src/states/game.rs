use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Color32},
    EguiContexts,
};

use super::AppState;

// Data structures

// Stores the key number (aka array index)
#[derive(Component)]
struct PianoKey(usize, PianoKeyType);

// The types of inputs on a MIDI keyboard
enum PianoKeyType {
    White,
    Black,
    // Slider,
    // Button,
}

// Plugin

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(game_setup.in_schedule(OnEnter(AppState::Game)))
            .add_system(spawn_piano.in_schedule(OnEnter(AppState::Game)))
            // Game loop
            .add_system(game_system.in_set(OnUpdate(AppState::Game)))
            // Cleanup
            .add_system(game_cleanup.in_schedule(OnExit(AppState::Game)));
    }
}

pub fn spawn_piano(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    const NUM_TOTAL_KEYS: usize = 61;
    const NUM_WHITE_KEYS: usize = 36;
    const NUM_BLACK_KEYS: usize = 25;
    const WHITE_KEY_WIDTH: f32 = 1.0;
    const WHITE_KEY_HEIGHT: f32 = 5.5;
    const WHITE_KEY_DEPTH: f32 = 0.25;
    const BLACK_KEY_WIDTH: f32 = 0.5;
    const BLACK_KEY_HEIGHT: f32 = 3.5;
    const BLACK_KEY_DEPTH: f32 = 0.5;

    // 0 = WHITE
    // 1 = BLACK
    const KEY_ORDER: [i32; 12] = [0, 1, 0, 1, 0, 0, 1, 0, 1, 0, 1, 0];

    // A set of keys is 12 (5 black, 7 white)
    let mut white_key_offset = 0;
    for index in 0..NUM_TOTAL_KEYS {
        let key_type_index = index % 12;
        let key_type_id = KEY_ORDER[key_type_index];
        let key_index = index as f32;
        let position_x = (white_key_offset as f32) * WHITE_KEY_WIDTH;

        // White key
        if key_type_id == 0 {
            println!("generating white key {}", key_index.to_string());
            // We get the position of white keys by incrementing an external offset
            // since we can't use the index of the loop
            white_key_offset += 1;

            // Spawn white piano keys
            commands.spawn((
                PianoKey(index, PianoKeyType::White),
                // Mesh
                PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Box::new(
                        WHITE_KEY_WIDTH,
                        WHITE_KEY_HEIGHT,
                        WHITE_KEY_DEPTH,
                    ))),
                    material: materials.add(Color::WHITE.into()),
                    transform: Transform::from_xyz(position_x, 0.0, 0.0),
                    ..default()
                },
            ));
        }

        // Black keys
        if key_type_id == 1 {
            println!("generating black key {}", key_index.to_string());
            let black_position_x = position_x + WHITE_KEY_WIDTH / 2.0;

            // Spawn white piano keys
            commands.spawn((
                PianoKey(index, PianoKeyType::Black),
                // Mesh
                PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Box::new(
                        BLACK_KEY_WIDTH,
                        BLACK_KEY_HEIGHT,
                        BLACK_KEY_DEPTH,
                    ))),
                    material: materials.add(Color::BLACK.into()),
                    transform: Transform::from_xyz(black_position_x, BLACK_KEY_HEIGHT / 4.0, 0.0),
                    ..default()
                },
            ));
        }
    }
}

pub fn game_setup(mut commands: Commands) {
    println!("Game setup");

    // Camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-30.0, 30.0, 100.0)
            .looking_at(Vec3::new(0.0, 10.0, 0.0), Vec3::Y),
        ..Default::default()
    });

    // Lighting
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
}

pub fn game_system() {}

pub fn game_cleanup() {
    println!("Game cleanup");
}
