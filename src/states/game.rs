use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Color32},
    EguiContexts,
};

use crate::{
    debug::DebugState,
    midi::{MidiEvents, MidiInputKey, MidiInputState},
};

use super::AppState;

// Data structures

// Distinguishes 3rd person camera entity
#[derive(Component)]
pub struct ThirdPersonCamera;

// Distinguishes a piano note entity (blocks going up from keyboard)
#[derive(Component)]
pub struct PianoNote(usize);

// Stores the input type in note to extend them
#[derive(Component)]
pub struct PianoNoteEvent(MidiEvents);

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

// Constants
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

#[derive(Component)]
pub struct TimelineNote;

#[derive(Resource)]
pub struct MusicTimelineState {
    current: usize,
    complete: bool,
}

pub struct MusicTimelineItem {
    // Time in seconds
    time: f32,
    // Note on keyboard
    note: u8,
    // How long note should be held down
    length: f32,
}

const MUSIC_TIMELINE: [MusicTimelineItem; 3] = [
    MusicTimelineItem {
        time: 1.0,
        note: 38,
        length: 3.0,
    },
    MusicTimelineItem {
        time: 2.0,
        note: 39,
        length: 3.0,
    },
    MusicTimelineItem {
        time: 3.0,
        note: 40,
        length: 3.0,
    },
];

// Plugin

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MusicTimelineState {
            current: 0,
            complete: false,
        })
        .add_system(game_setup.in_schedule(OnEnter(AppState::Game)))
        .add_system(spawn_piano.in_schedule(OnEnter(AppState::Game)))
        // Game loop
        .add_system(game_system.in_set(OnUpdate(AppState::Game)))
        .add_system(highlight_keys.in_set(OnUpdate(AppState::Game)))
        .add_system(spawn_music_notes.in_set(OnUpdate(AppState::Game)))
        .add_system(animate_music_notes.in_set(OnUpdate(AppState::Game)))
        .add_system(clear_music_notes.in_set(OnUpdate(AppState::Game)))
        .add_system(spawn_music_timeline.in_set(OnUpdate(AppState::Game)))
        .add_system(animate_music_timeline.in_set(OnUpdate(AppState::Game)))
        .add_system(check_timeline_collisions.in_set(OnUpdate(AppState::Game)))
        .add_system(debug_sync_camera.in_set(OnUpdate(AppState::Game)))
        // Cleanup
        .add_system(game_cleanup.in_schedule(OnExit(AppState::Game)));
    }
}

pub fn spawn_piano(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // A set of keys is 12 (5 black, 7 white)
    let mut white_key_offset = 0;
    for index in 0..NUM_TOTAL_KEYS {
        let key_type_index = index % 12;
        let key_type_id = KEY_ORDER[key_type_index];
        let key_index = index as f32;
        let position_x = (white_key_offset as f32) * WHITE_KEY_WIDTH;

        // White key
        if key_type_id == 0 {
            println!("[SETUP] Generating white key {}", key_index.to_string());
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
            println!("[SETUP] Generating black key {}", key_index.to_string());
            let black_position_x = position_x - WHITE_KEY_WIDTH / 2.0;

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

// Spawns notes on the music timeline
pub fn spawn_music_timeline(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    // piano_keys: Query<(&Transform, &PianoKeyId), With<PianoKey>>,
    midi_state: Res<MidiInputState>,
    mut timeline_state: ResMut<MusicTimelineState>,
    time: Res<Time>,
) {
    if timeline_state.complete {
        return;
    }

    let current_item = &MUSIC_TIMELINE[timeline_state.current];

    // We spawn
    if time.elapsed_seconds() >= current_item.time {
        println!("[TIMELINE] Spawning note");

        // Get the placement of piano key.
        // Key event index are multiplied by octaves, so we calculate actual index on piano.
        let octave_offset = get_octave(midi_state.octave) as u8;
        let real_index = current_item.note - octave_offset;
        let real_index_f32 = real_index as f32;
        let key_type_index = (real_index % 12) as usize;
        let key_type_id = KEY_ORDER[key_type_index];

        // We also have to account for black vs white keys
        // Count number of previous white keys to this key's position
        let num_white_keys = KEY_ORDER
            .iter()
            .enumerate()
            .filter(|(index, &key_type)| index < &(real_index as usize) && key_type == 0)
            .count() as f32;

        // Offset black keys slightly
        let position_x = if key_type_id == 0 {
            // White key
            num_white_keys
        } else {
            // Black key
            num_white_keys - WHITE_KEY_WIDTH / 2.0
        };

        commands.spawn((
            TimelineNote,
            PianoKeyId(current_item.note as usize),
            // Mesh
            PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Box::new(
                    BLACK_KEY_WIDTH,
                    BLACK_KEY_HEIGHT,
                    BLACK_KEY_DEPTH,
                ))),
                material: materials.add(Color::GREEN.into()),
                transform: Transform::from_xyz(position_x, 10.0, 0.0),
                ..default()
            },
        ));

        let next_index = timeline_state.current + 1;
        if MUSIC_TIMELINE.len() > next_index {
            timeline_state.current += 1;
        } else {
            timeline_state.complete = true;
        }
    }
}

pub fn animate_music_timeline(
    mut notes: Query<&mut Transform, With<TimelineNote>>,
    time: Res<Time>,
) {
    for mut note_position in notes.iter_mut() {
        let speed = time.delta().as_secs_f32() * 1.0;
        // Move down a little past keys
        note_position.translation.y -= speed;
    }
}

// Check for input events and change color of 3D piano keys
pub fn check_timeline_collisions(
    mut key_events: EventReader<MidiInputKey>,
    // midi_state: Res<MidiInputState>,
    notes: Query<(&Transform, &PianoKeyId), With<TimelineNote>>,
) {
    if key_events.is_empty() {
        return;
    }

    // Loop through key input events
    for key in key_events.iter() {
        // println!("[EVENTS] MidiInputKey {} {}", key.id, key.event.to_string());
        let check_id = key.id as usize;
        println!("[COLLISION] Key pressed...");

        // Loop through all the active notes on screen
        for (transform, id_component) in notes.iter() {
            let PianoKeyId(id) = id_component;
            println!("[COLLISION] Checking note ID {} vs {}", id, check_id);
            // Did the user hit a note floating around?
            if id == &check_id {
                println!("[COLLISION] Key pressed on note lane");

                if transform.translation.x <= WHITE_KEY_HEIGHT {
                    println!("[COLLISION] Key pressed in time or after");
                }
            }
        }
    }
}

// Check for input events and change color of 3D piano keys
pub fn spawn_music_notes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut key_events: EventReader<MidiInputKey>,
    piano_keys: Query<(&Transform, &PianoKeyId), With<PianoKey>>,
    mut music_notes: Query<(&PianoNote, &mut PianoNoteEvent)>,
    midi_state: Res<MidiInputState>,
) {
    if key_events.is_empty() {
        return;
    }

    let octave_offset = get_octave(midi_state.octave);

    // Loop through key input events
    for key in key_events.iter() {
        println!("[SPAWN] Music note - finding key");
        // Figure out the current octave offset

        // Determine input type (pressed vs released)
        match key.event {
            MidiEvents::Pressed | MidiEvents::Holding => {
                // Spawn key
                // Find key and get position
                let piano_key_result = piano_keys.iter().find(|(_, key_id_component)| {
                    if let PianoKeyId(key_id) = key_id_component {
                        let real_id = key_id + (octave_offset as usize);
                        // println!("[SPAWN] Music note - {} - {}", key_id, key.id);
                        real_id == (key.id as usize)
                    } else {
                        false
                    }
                });

                if let Some((piano_key_transform, _)) = piano_key_result {
                    // println!("[SPAWN] Music note - spawned");
                    let note_id = (key.id as i32) - octave_offset;
                    // Spawn note where key is
                    commands.spawn((
                        PianoNote(note_id as usize),
                        PianoNoteEvent(MidiEvents::Pressed),
                        // Mesh
                        PbrBundle {
                            mesh: meshes.add(Mesh::from(shape::Box::new(
                                WHITE_KEY_WIDTH,
                                0.5,
                                -WHITE_KEY_DEPTH,
                            ))),
                            material: materials.add(Color::TEAL.into()),
                            transform: Transform::from_xyz(
                                piano_key_transform.translation.x,
                                piano_key_transform.translation.y + WHITE_KEY_HEIGHT / 2.0,
                                piano_key_transform.translation.z,
                            ),
                            ..default()
                        },
                    ));
                }
            }
            MidiEvents::Released => {
                // Mark key as released
                // We loop through all the notes and match ID to key event's ID
                for (id_component, mut event_component) in music_notes.iter_mut() {
                    let PianoNote(id) = id_component;
                    let PianoNoteEvent(mut event) = event_component.as_mut();
                    let real_id = id + (octave_offset as usize);
                    if real_id == (key.id as usize) {
                        if MidiEvents::Pressed == event {
                            event = MidiEvents::Released;
                            *event_component = PianoNoteEvent(event);
                        }
                    }
                }
            }
        }
    }
}

pub fn animate_music_notes(
    mut notes: Query<(&mut Transform, &PianoNoteEvent), With<PianoNote>>,
    time: Res<Time>,
) {
    let animation_speed = 5.0;
    let animation_delta = time.delta().as_secs_f32() * animation_speed;

    for (mut note, key_type_component) in notes.iter_mut() {
        let PianoNoteEvent(key_type) = key_type_component;
        if key_type == &MidiEvents::Pressed {
            let scale_speed = 5.0;
            let scale_delta = time.delta().as_secs_f32() * scale_speed;
            // Scale up gradually
            note.scale.y += scale_delta;
            note.translation.y += animation_delta / 3.0;
        } else {
            // Move up
            note.translation.y += animation_delta;
        }
    }
}

pub fn clear_music_notes(
    mut commands: Commands,
    notes: Query<(Entity, &Transform), With<PianoNote>>,
) {
    for (entity, note) in notes.iter() {
        if note.translation.y > 100.0 {
            commands.entity(entity).despawn();
        }
    }
}

// Check for input events and change color of 3D piano keys
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

    // Loop through key input events
    for key in key_events.iter() {
        // println!("[EVENTS] MidiInputKey {} {}", key.id, key.event.to_string());

        // Figure out the current octave offset
        let octave_offset = get_octave(midi_state.octave);

        // Select the right key and highlight it
        for (entity, key_id_component, key_type) in &key_entities {
            let PianoKeyId(key_id) = key_id_component;
            // Get the "real" key ID
            // We store keys from 0 to total, but MIDI outputs it relative to octave
            // So we do the math to "offset" the keys to match MIDI output
            let real_id = key_id + (octave_offset as usize);
            let check_id = key.id as usize;

            if real_id == check_id {
                // println!(
                //     "[EVENTS] Highlighting key {} {}",
                //     key.id,
                //     key.event.to_string()
                // );

                // Change color of the selected key
                // To get a material from a specific entity we grab it's "Handle"
                // then use that with a "Resource" to get the actual material
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
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(17.5, -24.7, 29.5)
                .looking_at(Vec3::new(17.5, 26.3, 0.0), Vec3::Y),
            ..Default::default()
        },
        ThirdPersonCamera,
    ));

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

pub fn debug_sync_camera(
    mut cameras: Query<(&mut Transform, &ThirdPersonCamera), Without<PianoKey>>,
    debug_state: Res<DebugState>,
) {
    if let Ok((mut camera, _)) = cameras.get_single_mut() {
        camera.translation.x = debug_state.debug_position.x;
        camera.translation.y = debug_state.debug_position.y;
        camera.translation.z = debug_state.debug_position.z;

        camera.look_at(debug_state.camera_look, Vec3::Y);

        // Sync rotation
        camera.rotation.x = debug_state.rotation.x;
        camera.rotation.y = debug_state.rotation.y;
        camera.rotation.z = debug_state.rotation.z;
    }
}

pub fn game_system() {}

pub fn game_cleanup() {
    println!("Game cleanup");
}

// Utility functions
fn get_octave(current_octave: i32) -> i32 {
    // Figure out the current octave
    // My Arturia Keylab 61 starts at "0" octave and ranges from -3 to 3
    // So this number may differ based on total number of keys
    let octave = 3 - current_octave;
    let octave_offset = octave * 12;
    octave_offset
}

fn lerp(start: &f32, end: &f32, amt: f32) -> f32 {
    return (1.0 - amt) * start + amt * end;
}
