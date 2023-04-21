use std::time::Duration;

use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Color32},
    EguiContexts,
};
use bevy_rapier3d::prelude::*;

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

// The whole piano entity (parent of the piano keys)
#[derive(Component)]
pub struct Piano;

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

// The Floor entity. Used to filter some collision events.
#[derive(Component)]
pub struct Floor;

// The Enemy entity. Used to filter some collision events.
#[derive(Component)]
pub struct Enemy {
    name: String,
    score: i32,
    destroy: bool,
    timer: Option<Timer>,
}

// Events

// Notes collided with enemy
pub struct EnemyColliderEvent(Entity);

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

// Plugin

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<EnemyColliderEvent>()
            .add_system(game_setup.in_schedule(OnEnter(AppState::Game)))
            .add_system(spawn_piano.in_schedule(OnEnter(AppState::Game)))
            .add_system(spawn_enemies.in_schedule(OnEnter(AppState::Game)))
            // Game loop
            .add_system(game_system.in_set(OnUpdate(AppState::Game)))
            .add_system(highlight_keys.in_set(OnUpdate(AppState::Game)))
            .add_system(spawn_music_notes.in_set(OnUpdate(AppState::Game)))
            .add_system(animate_music_notes.in_set(OnUpdate(AppState::Game)))
            .add_system(clear_music_notes.in_set(OnUpdate(AppState::Game)))
            .add_system(display_collision_events.in_set(OnUpdate(AppState::Game)))
            .add_system(mark_enemy_for_destruction.in_set(OnUpdate(AppState::Game)))
            .add_system(enemy_destruction_animation.in_set(OnUpdate(AppState::Game)))
            .add_system(enemy_destruction_despawn.in_set(OnUpdate(AppState::Game)))
            // .add_system(check_collisions_manual.in_set(OnUpdate(AppState::Game)))
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
    // 0 = WHITE
    // 1 = BLACK
    const KEY_ORDER: [i32; 12] = [0, 1, 0, 1, 0, 0, 1, 0, 1, 0, 1, 0];

    commands
        .spawn((
            Piano,
            SpatialBundle::from_transform(Transform::from_xyz(0.0, 25.0, 0.0)),
        ))
        .with_children(|children| {
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
                    children.spawn((
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
                    children.spawn((
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
                            transform: Transform::from_xyz(
                                black_position_x,
                                BLACK_KEY_HEIGHT / 4.0,
                                0.0,
                            ),
                            ..default()
                        },
                    ));
                }
            }
        });
}

// Check for input events and change color of 3D piano keys
pub fn spawn_music_notes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut key_events: EventReader<MidiInputKey>,
    piano_keys: Query<(&GlobalTransform, &PianoKeyId), With<PianoKey>>,
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
                    let transform = piano_key_transform.compute_transform();
                    println!(
                        "[SPAWN] Music note - spawned {} {}",
                        transform.translation.x, transform.translation.y
                    );
                    let note_id = (key.id as i32) - octave_offset;
                    // Spawn note where key is
                    commands.spawn((
                        PianoNote(note_id as usize),
                        PianoNoteEvent(MidiEvents::Pressed),
                        // Physics
                        Collider::cuboid(WHITE_KEY_WIDTH, 0.5, WHITE_KEY_DEPTH),
                        // Needed to detect collision events
                        // ActiveEvents::COLLISION_EVENTS,
                        Velocity {
                            linvel: Vec3 {
                                x: 0.0,
                                y: -((key.intensity as f32) / 100.0),
                                z: 0.0,
                            },
                            angvel: Vec3::ZERO,
                        },
                        ContactForceEventThreshold(30.0),
                        RigidBody::Dynamic,
                        // Debug without mesh
                        // SpatialBundle::from_transform(
                        //     Transform::from_xyz(
                        //         transform.translation.x,
                        //         transform.translation.y - WHITE_KEY_HEIGHT,
                        //         transform.translation.z,
                        //     )
                        // )
                        // Mesh
                        PbrBundle {
                            mesh: meshes.add(Mesh::from(shape::Box::new(
                                WHITE_KEY_WIDTH,
                                0.5,
                                WHITE_KEY_DEPTH,
                            ))),
                            material: materials.add(Color::TEAL.into()),
                            transform: Transform::from_xyz(
                                transform.translation.x,
                                transform.translation.y - WHITE_KEY_HEIGHT,
                                transform.translation.z,
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
    mut notes: Query<(&mut Transform, &mut Velocity, &PianoNoteEvent), With<PianoNote>>,
    time: Res<Time>,
) {
    let animation_speed = 1.0;
    // let animation_delta = time.delta().as_secs_f32() * animation_speed;
    let animation_delta = animation_speed;

    for (mut note, mut velocity, key_type_component) in notes.iter_mut() {
        let PianoNoteEvent(key_type) = key_type_component;
        if key_type == &MidiEvents::Pressed {
            let scale_speed = 5.0;
            let scale_delta = time.delta().as_secs_f32() * scale_speed;
            velocity.linvel = Vec3::new(0.0, 0.001, 0.0);
            // Scale up gradually
            note.scale.y += scale_delta;
            // note.translation.y += animation_delta / 3.0;
            // velocity.linvel.y += animation_delta / 3.0;
            // velocity.linvel.y -= animation_delta / 3.0;
            // velocity.linvel.x -= animation_delta / 3.0;
        } else {
            // Move up
            // note.translation.y += animation_delta;
            // velocity.linvel.y += animation_delta;
            // velocity.linvel -= Vec3::new(0.0, 1.0, 0.0);
            // velocity.linvel.y -= animation_delta;
            // velocity.linvel.x -= animation_delta;
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

pub fn spawn_enemies(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let enemy_size = 0.5;
    commands.spawn((
        Enemy {
            name: "Test enemy 1".to_string(),
            score: 100,
            destroy: false,
            timer: None,
        },
        Collider::cuboid(enemy_size, enemy_size, enemy_size),
        ColliderDebugColor(Color::hsl(220.3, 1.0, 220.3)),
        // Needed to detect collision events
        ActiveEvents::COLLISION_EVENTS,
        PbrBundle {
            mesh: meshes.add(shape::Box::new(enemy_size, enemy_size, enemy_size).into()),
            material: materials.add(Color::hex("#DDDDDD").unwrap().into()),
            transform: Transform::from_xyz(10.0, 15.0, 0.0),
            ..default()
        },
    ));

    commands.spawn((
        Enemy {
            name: "Test enemy 2".to_string(),
            score: 100,
            destroy: false,
            timer: None,
        },
        Collider::cuboid(enemy_size, enemy_size, enemy_size),
        ColliderDebugColor(Color::hsl(220.3, 1.0, 220.3)),
        // Needed to detect collision events
        ActiveEvents::COLLISION_EVENTS,
        PbrBundle {
            mesh: meshes.add(shape::Box::new(enemy_size, enemy_size, enemy_size).into()),
            material: materials.add(Color::hex("#DDDDDD").unwrap().into()),
            transform: Transform::from_xyz(30.0, 10.0, -enemy_size),
            ..default()
        },
    ));
}

pub fn game_setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
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
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(8.0, 8.0, 4.0),
        ..default()
    });

    // Floor / ground
    let ground_size = 200.1;
    let ground_height = 0.01;

    commands.spawn((
        Floor,
        Collider::cuboid(ground_size, ground_height, ground_size),
        PbrBundle {
            mesh: meshes.add(shape::Plane::from_size(ground_size).into()),
            material: materials.add(Color::hex("#DDDDDD").unwrap().into()),
            transform: Transform::from_xyz(0.0, -ground_height, 0.0),
            ..default()
        },
    ));
}

fn display_collision_events(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    mut contact_force_events: EventReader<ContactForceEvent>,
    mut enemy_collider_event: EventWriter<EnemyColliderEvent>, // mut attach_events: EventWriter<AttachObjectEvent>,
                                                               // player_entity: Query<Entity, With<Player>>,
                                                               // floor_entity: Query<Entity, With<Floor>>,
) {
    // Check for collisions
    for collision_event in collision_events.iter() {
        match collision_event {
            CollisionEvent::Started(first_entity, second_entity, _) => {
                println!(
                    "{} collided with {}",
                    first_entity.index(),
                    second_entity.index()
                );
                enemy_collider_event.send(EnemyColliderEvent(*first_entity));
                // commands.entity(*first_entity).insert((
                //     RigidBody::Dynamic,
                //     Velocity {
                //         linvel: Vec3::new(0.0, -1.0, 0.0),
                //         angvel: Vec3::ZERO,
                //     },
                // ));
                // commands.entity(*first_entity).insert(RigidBody::Dynamic);
                // commands.entity(*second_entity).insert(RigidBody::Dynamic);
            }
            CollisionEvent::Stopped(first_entity, second_entity, event) => {}
        }
    }

    for contact_force_event in contact_force_events.iter() {
        println!("Received contact force event: {contact_force_event:?}");
    }
}

pub fn mark_enemy_for_destruction(
    mut collider_events: EventReader<EnemyColliderEvent>,
    mut enemies: Query<&mut Enemy>,
) {
    if !collider_events.is_empty() {
        // We loop over all events and use the event's collider entity index
        for event in collider_events.iter() {
            let EnemyColliderEvent(enemy_entity) = event;

            let mut enemy_data = enemies.get_mut(*enemy_entity).unwrap();

            enemy_data.destroy = true;
            enemy_data.timer = Some(Timer::from_seconds(2.0, TimerMode::Once));
        }
    }
}

pub fn enemy_destruction_animation(
    mut enemies: Query<(&mut Enemy, &mut Transform)>,
    time: Res<Time>,
) {
    for (mut enemy, mut enemy_position) in enemies.iter_mut() {
        if enemy.destroy {
            let mut timer = enemy.timer.as_mut().unwrap();
            timer.tick(time.delta());
            let elapsed = timer.elapsed_secs();
            enemy_position.rotate_y(elapsed * 3.0);
        }
    }
}

pub fn enemy_destruction_despawn(
    mut commands: Commands,
    mut enemies: Query<(&mut Enemy, Entity)>,
    time: Res<Time>,
) {
    for (mut enemy, enemy_entity) in enemies.iter_mut() {
        if enemy.destroy {
            let mut timer = enemy.timer.as_mut().unwrap();
            timer.tick(time.delta());

            if timer.finished() {
                commands.entity(enemy_entity).despawn();
            }
        }
    }
}

pub fn check_collisions_manual(
    enemies: Query<&Transform, With<Enemy>>,
    notes: Query<&Transform, With<PianoNote>>,
) {
    for enemy in enemies.iter() {
        println!(
            "[ENEMY] Position: {} {}",
            enemy.translation.x, enemy.translation.y
        );
    }
    for note in notes.iter() {
        println!(
            "[NOTE] Position: {} {}",
            note.translation.x, note.translation.y
        );
    }
}

pub fn debug_sync_camera(
    mut cameras: Query<(&mut Transform, &ThirdPersonCamera)>,
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

fn get_octave(current_octave: i32) -> i32 {
    // Figure out the current octave
    // My Arturia Keylab 61 starts at "0" octave and ranges from -3 to 3
    // So this number may differ based on total number of keys
    let octave = 3 - current_octave;
    let octave_offset = octave * 12;
    octave_offset
}
