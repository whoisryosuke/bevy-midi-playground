use std::time::Duration;

use bevy::prelude::*;
use bevy_egui::{
    egui::{self, epaint, Color32},
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

// The Y coordinate of where notes start and stop
const TIMELINE_TOP: f32 = 30.0;
const TIMELINE_BOTTOM: f32 = 0.0;
// The length of time our "track" represents. Total note travel length across screen.
const TIMELINE_LENGTH: f32 = 10.0;
const TIMELINE_TOTAL_TIME: f32 = 30.0;

#[derive(Component)]
pub struct TimelineNote;

#[derive(Component)]
pub struct TimelineNoteTime(f32);

#[derive(Resource)]
pub struct GameState {
    score: i32,
}

#[derive(Resource)]
pub struct MusicTimelineState {
    current: usize,
    playing: bool,
    complete: bool,
    timer: Timer,
}

pub struct MusicTimelineItem {
    // Time in seconds
    time: f32,
    // Note on keyboard
    note: u8,
    // How long note should be held down
    length: f32,
}

#[derive(Resource)]
pub struct MusicTimeline {
    // timeline: Vec<MusicTimelineItem>,
    timeline: [MusicTimelineItem; 3],
    total_time: f32,
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
        app.insert_resource(GameState { score: 0 })
            .insert_resource(MusicTimelineState {
                current: 0,
                playing: false,
                complete: false,
                timer: Timer::from_seconds(0.0, TimerMode::Once),
            })
            .insert_resource(MusicTimeline {
                timeline: MUSIC_TIMELINE,
                total_time: TIMELINE_TOTAL_TIME,
            })
            .add_system(game_setup.in_schedule(OnEnter(AppState::Game)))
            .add_system(spawn_piano.in_schedule(OnEnter(AppState::Game)))
            // Game loop
            .add_system(game_system.in_set(OnUpdate(AppState::Game)))
            .add_system(highlight_keys.in_set(OnUpdate(AppState::Game)))
            // .add_system(spawn_music_notes.in_set(OnUpdate(AppState::Game)))
            // .add_system(animate_music_notes.in_set(OnUpdate(AppState::Game)))
            // .add_system(clear_music_notes.in_set(OnUpdate(AppState::Game)))
            .add_system(spawn_music_timeline.in_set(OnUpdate(AppState::Game)))
            .add_system(animate_music_timeline.in_set(OnUpdate(AppState::Game)))
            .add_system(check_timeline_collisions.in_set(OnUpdate(AppState::Game)))
            .add_system(clear_complete_timeline_notes.in_set(OnUpdate(AppState::Game)))
            .add_system(score_ui.in_set(OnUpdate(AppState::Game)))
            .add_system(debug_sync_camera.in_set(OnUpdate(AppState::Game)))
            .add_system(debug_game_ui.in_set(OnUpdate(AppState::Game)))
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
    if timeline_state.timer.elapsed_secs() >= current_item.time {
        println!("[TIMELINE] Spawning note");

        // Get the placement of piano key.
        // Key event index are multiplied by octaves, so we calculate actual index on piano.
        let octave_offset = get_octave(midi_state.octave) as u8;
        let real_index = current_item.note - octave_offset;
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

        let shape = if key_type_id == 0 {
            shape::Box::new(WHITE_KEY_WIDTH, WHITE_KEY_HEIGHT, WHITE_KEY_DEPTH)
        } else {
            shape::Box::new(BLACK_KEY_WIDTH, BLACK_KEY_HEIGHT, BLACK_KEY_DEPTH)
        };

        commands.spawn((
            TimelineNote,
            TimelineNoteTime(current_item.time),
            PianoKeyId(current_item.note as usize),
            // Mesh
            PbrBundle {
                mesh: meshes.add(Mesh::from(shape)),
                material: materials.add(Color::GREEN.into()),
                transform: Transform::from_xyz(position_x, TIMELINE_TOP, 0.0),
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
    mut notes: Query<(&mut Transform, &TimelineNoteTime), With<TimelineNote>>,
    time: Res<Time>,
    mut timeline_state: ResMut<MusicTimelineState>,
) {
    timeline_state.timer.tick(time.delta());
    let current_time = timeline_state.timer.elapsed().as_secs_f32();
    for (mut note_position, start_time_component) in notes.iter_mut() {
        let TimelineNoteTime(start_time) = start_time_component;
        // Current time represents top of the screen
        // Get a number from 0 - TIMELINE_LENGTH to
        // proportionally calculate actual distance from the time
        let note_difference = start_time - current_time;
        let note_distance = note_difference * TIMELINE_TOP / TIMELINE_LENGTH;

        note_position.translation.y = TIMELINE_TOP + note_distance;
    }
}

// Check for input events and change color of 3D piano keys
pub fn check_timeline_collisions(
    mut commands: Commands,
    mut key_events: EventReader<MidiInputKey>,
    // midi_state: Res<MidiInputState>,
    notes: Query<(Entity, &Transform, &PianoKeyId, &TimelineNoteTime), With<TimelineNote>>,
    timeline_state: Res<MusicTimelineState>,
    mut game_state: ResMut<GameState>,
) {
    if key_events.is_empty() {
        return;
    }

    // Loop through key input events
    for key in key_events.iter() {
        // println!("[EVENTS] MidiInputKey {} {}", key.id, key.event.to_string());
        let check_id = key.id as usize;

        // Loop through all the active notes on screen
        for (entity, transform, id_component, note_time_component) in notes.iter() {
            let PianoKeyId(id) = id_component;
            let TimelineNoteTime(note_time) = note_time_component;
            // println!("[COLLISION] Checking note ID {} vs {}", id, check_id);
            // Did the user hit a note floating around?
            if id == &check_id {
                println!("[COLLISION] Key pressed on note lane {}", &id);

                // @TODO: Add a "buffer"/offset above key height to help player
                if transform.translation.y <= WHITE_KEY_HEIGHT {
                    println!(
                        "[COLLISION] Key pressed in time or after {} - {} = {}",
                        transform.translation.y,
                        WHITE_KEY_HEIGHT,
                        WHITE_KEY_HEIGHT - transform.translation.y
                    );
                    // Accuracy is determined by the placement of the note when user pressed key
                    // We divide by 5 because that's the max distance the user can make a mistake.
                    // So we get a percentage of how bad they did from 0 - 5.
                    let accuracy = (WHITE_KEY_HEIGHT - transform.translation.y) / 5.0;

                    // Since the accuracy goes from 0.0 (super accurate) to 1.0 (not as much)
                    // We find the percent of score to remove based on accuracy (e.g. score * 0.5)
                    // then we subtract from initial score.
                    let initial_score = 1000;

                    let mistake_cost = (initial_score as f32 * accuracy) as i32;
                    let mistake_cost = if mistake_cost < 0 { 0 } else { mistake_cost };

                    let score = initial_score - mistake_cost;
                    println!("adding score {}", score);

                    // Update game state with the new score
                    game_state.score += score;

                    // Destroy the note immediately
                    // @TODO: Instead...mark it for destruction - animate it away
                    commands.entity(entity).despawn_recursive();

                    // Check time for debug purposes
                    let current_time = timeline_state.timer.elapsed_secs();
                    println!(
                        "[COLLISION] User time: {} - Note time: {}",
                        current_time, note_time,
                    );
                }
            }
        }
    }
}

// Check for input events and change color of 3D piano keys
pub fn clear_complete_timeline_notes(
    mut commands: Commands,
    notes: Query<(&Transform, Entity), With<TimelineNote>>,
) {
    // Loop through all the active notes on screen
    for (note_transform, note_entity) in notes.iter() {
        if note_transform.translation.y <= TIMELINE_BOTTOM {
            commands.entity(note_entity).despawn();
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

fn score_ui(mut contexts: EguiContexts, game_state: Res<GameState>) {
    // Set window styles
    let ctx = contexts.ctx_mut();
    // let old = ctx.style().visuals.clone();
    // ctx.set_visuals(egui::Visuals {
    //     window_fill: Color32::TRANSPARENT,
    //     panel_fill: Color32::TRANSPARENT,
    //     window_stroke: egui::Stroke {
    //         color: Color32::TRANSPARENT,
    //         width: 0.0,
    //     },
    //     window_shadow: epaint::Shadow {
    //         color: Color32::TRANSPARENT,
    //         ..old.window_shadow
    //     },
    //     ..old
    // });

    // Create window + UI
    egui::Window::new("Score").title_bar(false).show(ctx, |ui| {
        ui.label("Score");
        ui.heading(game_state.score.to_string());
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

fn debug_game_ui(
    mut contexts: EguiContexts,
    mut timeline_state: ResMut<MusicTimelineState>,
    mut game_state: ResMut<GameState>,
    timeline: Res<MusicTimeline>,
    time: Res<Time>,
) {
    timeline_state.timer.tick(time.delta());

    if timeline_state.timer.finished() {
        timeline_state.complete = true;
        timeline_state.playing = false;
    }

    egui::Window::new("Debug Game State").show(contexts.ctx_mut(), |ui| {
        ui.horizontal(|ui| {
            ui.heading("Timer");
            ui.label(timeline_state.timer.elapsed().as_secs_f32().to_string())
        });

        let complete_text = if timeline_state.complete {
            "Complete".to_string()
        } else {
            "Not Complete".to_string()
        };
        ui.heading(complete_text);

        let playing_text = if timeline_state.playing {
            "Playing".to_string()
        } else {
            "Not Playing".to_string()
        };
        ui.heading(playing_text);

        if !timeline_state.playing {
            if ui.button("Start").clicked() {
                timeline_state.complete = false;
                timeline_state.playing = true;
                // timeline_state
                //     .timer
                //     .set_duration(Duration::from_secs_f32(timeline.total_time));
                // timeline_state.timer.reset();
                // timeline_state.timer.unpause();
                timeline_state.timer = Timer::new(
                    Duration::from_secs_f32(timeline.total_time),
                    TimerMode::Once,
                );
            }

            if timeline_state.timer.paused() {
                if ui.button("Unpause").clicked() {
                    timeline_state.timer.unpause();
                }
            }
        } else {
            if ui.button("Pause").clicked() {
                timeline_state.playing = false;
                timeline_state.timer.pause();
            }
        }

        if ui.button("Reset").clicked() {
            timeline_state.playing = false;
            timeline_state.current = 0;
            timeline_state.timer.reset();
            timeline_state.timer.pause();

            game_state.score = 0;

            // @TODO: Add a reset event or flag so the game can
            // clear any 3D notes before starting new scene
        }
    });
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
