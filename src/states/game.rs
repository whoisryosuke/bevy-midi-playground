use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Color32},
    EguiContexts,
};

use crate::midi::{MidiInputKey, MidiInputState};

use super::AppState;

// Data structures

// Distinguishes a piano key entity
#[derive(Component)]
pub struct PianoKey;

// The index of a key (0 to total number of keys)
#[derive(Component)]
pub struct PianoKeyId(usize);

// The type of key (black, white, etc)
// The types of inputs on a MIDI keyboard
#[derive(Component)]
pub enum PianoKeyType {
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
            .add_system(highlight_keys.in_set(OnUpdate(AppState::Game)))
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
                PianoKey,
                PianoKeyId(index),
                PianoKeyType::White,
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
                PianoKey,
                PianoKeyId(index),
                PianoKeyType::Black,
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

pub fn highlight_keys(
    mut key_events: EventReader<MidiInputKey>,
    midi_state: Res<MidiInputState>,
    key_entities: Query<(Entity, &PianoKeyId, &PianoKeyType), With<PianoKey>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut key_materials: Query<&mut Handle<StandardMaterial>>,
    // mut assets: Assets<StandardMaterial>,
) {
    if key_events.is_empty() {
        return;
    }

    for key in key_events.iter() {
        // println!("[EVENTS] MidiInputKey {} {}", key.id, key.event.to_string());
        let octave = 3 - midi_state.octave;
        let octave_offset = octave * 12;
        println!("octave {} {}", &octave, &octave_offset);

        // Select the right key and highlight it
        for (entity, key_id_component, key_type) in &key_entities {
            let PianoKeyId(key_id) = key_id_component;
            // Get the "real" key ID
            // We store keys from 0 to total, but MIDI outputs it relative to octave
            // So we do the math to "offset" the keys to match MIDI output
            let real_id = key_id + (octave_offset as usize);
            let check_id = key.id as usize;

            println!("checking keys {} and {}", &real_id, &check_id);

            if real_id == check_id {
                println!(
                    "[EVENTS] Highlighting key {} {}",
                    key.id,
                    key.event.to_string()
                );

                if let Ok(handle) = key_materials.get_mut(entity) {
                    if let Some(material) = materials.get_mut(&handle) {
                        let color: Color;
                        match key.event {
                            crate::midi::MidiEvents::Pressed => {
                                color = Color::BLUE;
                            }
                            crate::midi::MidiEvents::Released => match key_type {
                                PianoKeyType::White => {
                                    color = Color::WHITE;
                                }
                                PianoKeyType::Black => {
                                    color = Color::BLACK;
                                }
                            },
                            crate::midi::MidiEvents::Holding => {
                                color = Color::BLUE;
                            }
                        }
                        material.base_color = color.into();
                    }
                }
            }
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
