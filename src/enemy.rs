use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use rand::Rng;

use crate::states::game::{PianoKey, PianoKeyType, WHITE_KEY_WIDTH};
use crate::states::AppState;
// Resources

pub struct EnemyMove {
    movement: Vec2,
    start_time: f32,
}

// The Enemy entity. Used to filter some collision events.
#[derive(Component)]
pub struct Enemy {
    name: String,
    score: i32,
    destroy: bool,
    timer: Option<Timer>,
    next_move: Option<EnemyMove>,
}

#[derive(Component)]
pub struct EnemyProjectile;

const ENEMY_SPAWN_TIME: f32 = 3.0;
const ENEMY_MAX_COUNT: i32 = 2;
const ENEMY_SIZE: f32 = 0.5;
const ENEMY_MOVE_TIME: f32 = 0.1;
const ENEMY_DEATH_TIME: f32 = 0.5;
// Projectiles
const ENEMY_SHOOT_TIMER_MIN: f32 = 1.0;
const ENEMY_SHOOT_TIMER_MAX: f32 = 3.0;
const ENEMY_SHOT_SIZE: f32 = 0.25;

#[derive(Resource)]
pub struct EnemyState {
    // Number of total enemies
    count: i32,
    // Timer between spawning enemies
    spawn_timer: Timer,
}

// Events

// Notes collided with enemy
pub struct EnemyColliderEvent(pub Entity);

// Plugin

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<EnemyColliderEvent>()
            .insert_resource(EnemyState {
                count: 0,
                spawn_timer: Timer::from_seconds(ENEMY_SPAWN_TIME, TimerMode::Once),
            })
            // Startup
            // .add_system(spawn_enemies.in_schedule(OnEnter(AppState::Game)))
            // Game loop
            .add_system(enemy_spawn_manager.in_set(OnUpdate(AppState::Game)))
            .add_system(mark_enemy_for_destruction.in_set(OnUpdate(AppState::Game)))
            .add_system(enemy_destruction_animation.in_set(OnUpdate(AppState::Game)))
            .add_system(enemy_animation.in_set(OnUpdate(AppState::Game)))
            .add_system(enemy_shooting.in_set(OnUpdate(AppState::Game)))
            .add_system(enemy_projectile_animation.in_set(OnUpdate(AppState::Game)))
            .add_system(detect_enemy_collision.in_set(OnUpdate(AppState::Game)))
            // Cleanup
            .add_system(enemy_cleanup.in_schedule(OnExit(AppState::Game)));
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

            // Get the enemy data using the entity from event
            let mut enemy_data = enemies.get_mut(*enemy_entity).unwrap();

            // Set it to destroy and create new internal timer
            enemy_data.destroy = true;
            enemy_data.timer = Some(Timer::from_seconds(ENEMY_DEATH_TIME, TimerMode::Once));
        }
    }
}

pub fn enemy_destruction_animation(
    mut commands: Commands,
    mut enemies: Query<(&mut Enemy, &mut Transform, Entity)>,
    time: Res<Time>,
    mut enemy_state: ResMut<EnemyState>,
) {
    for (mut enemy, mut enemy_position, enemy_entity) in enemies.iter_mut() {
        if enemy.destroy {
            let mut timer = enemy.timer.as_mut().unwrap();
            // Tick the timer (necessary)
            timer.tick(time.delta());

            // Animate the enemy rotation using the timer
            let elapsed = timer.elapsed_secs();
            enemy_position.rotate_y(elapsed * 3.0);

            // Despawn if timer is done
            if timer.finished() {
                commands.entity(enemy_entity).despawn();

                enemy_state.count -= 1;
            }
        }
    }
}

// Handles spawning new enemies if count isn't high enough
fn enemy_spawn_manager(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut enemy_state: ResMut<EnemyState>,
) {
    while enemy_state.count < ENEMY_MAX_COUNT {
        // Get a random position
        // We want between X: ~10-30 // Y: ~15-5

        let mut rng = rand::thread_rng();
        let position_x = rng.gen_range(10.0..30.0);
        let position_y = rng.gen_range(5.0..15.0);

        commands.spawn((
            Enemy {
                name: "Test enemy".to_string(),
                score: 100,
                destroy: false,
                timer: None,
                next_move: None,
            },
            Collider::cuboid(ENEMY_SIZE, ENEMY_SIZE, ENEMY_SIZE),
            ColliderDebugColor(Color::hsl(220.3, 1.0, 220.3)),
            // Needed to detect collision events
            ActiveEvents::COLLISION_EVENTS,
            PbrBundle {
                mesh: meshes.add(shape::Box::new(ENEMY_SIZE, ENEMY_SIZE, ENEMY_SIZE).into()),
                material: materials.add(Color::hex("#DDDDDD").unwrap().into()),
                transform: Transform::from_xyz(position_x, position_y, 0.0),
                ..default()
            },
        ));

        enemy_state.count += 1;
    }
}

fn generate_new_move(start_time: f32, initial_position: &Vec3) -> Option<EnemyMove> {
    let mut rng = rand::thread_rng();
    let direction = rng.gen_range(-1..1) as f32;
    let direction = if direction == 0.0 { 1.0 } else { direction };
    let random_x = rng.gen_range(0.1..1.0);
    let random_y = rng.gen_range(0.05..0.5);
    let position_x = initial_position.x + (random_x * direction);
    let position_y = initial_position.y + (random_y * direction);
    Some(EnemyMove {
        movement: Vec2::new(position_x, position_y),
        start_time,
    })
}

fn enemy_animation(mut enemies: Query<(&mut Transform, &mut Enemy)>, time: Res<Time>) {
    for (mut enemy_position, mut enemy_data) in enemies.iter_mut() {
        // Check if it has a next move
        if enemy_data.next_move.is_none() {
            // Generate a new move
            // let direction = rng.gen_range(-1..1);
            // Remove zero from the equation
            // let direction = if direction == 0 { 1 } else { direction };
            // let speed_x = rng.gen_range(1.0..10.0);
            // let speed_y = rng.gen_range(1.0..3.0);
            // let position_x = (direction as f32) * speed_x;
            // let position_y = (direction as f32) * speed_y;

            // Check limit
            // Limit of X is 10 to 30
            // let position_x = position_x.min(10.0).max(30.0);
            // let position_y = position_y.min(5.0).max(15.0);

            // enemy_data.next_move = Some(EnemyMove {
            //     movement: Vec2::new(position_x, position_y),
            //     start_time: time.elapsed_seconds(),
            // });

            enemy_data.next_move =
                generate_new_move(time.elapsed_seconds(), &enemy_position.translation);
        }

        // Done? Next move
        if let Some(enemy_move) = &mut enemy_data.next_move {
            let time_delta = time.elapsed_seconds() - enemy_move.start_time;
            // Longer than animation time? New move
            if time_delta > ENEMY_MOVE_TIME {
                enemy_data.next_move =
                    generate_new_move(time.elapsed_seconds(), &enemy_position.translation);
            }
        }

        // Animate otherwise
        if let Some(enemy_move) = &enemy_data.next_move {
            let time_delta = time.elapsed_seconds() - enemy_move.start_time;

            // Calculate rate of range
            // We want enemies to move relative to the movement
            // So bigger moves = longer time to move
            // 3 seconds - 2 seconds = 1 second
            // 30 / 10 = 3 * 2 = 6
            // let rate_of_change = (enemy_move.movement.x / 10.0) * 2.0;
            let movement_speed = time_delta / ENEMY_MOVE_TIME;
            enemy_position.translation = enemy_position.translation.lerp(
                Vec3::new(
                    enemy_move.movement.x,
                    enemy_move.movement.y,
                    enemy_position.translation.z,
                ),
                movement_speed,
            );
            // enemy_position.translation.x += 1.0;
        }
    }
}

fn enemy_shooting(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut enemies: Query<(&mut Enemy, &Transform)>,
    time: Res<Time>,
) {
    for (mut enemy, enemy_position) in enemies.iter_mut() {
        // Marked for destruction? Ignore it.
        if enemy.destroy {
            return;
        }

        match &mut enemy.timer {
            Some(timer) => {
                // Tick the timer
                timer.tick(time.delta());

                if timer.finished() {
                    // Shoot
                    println!("[PROJECTILE] enemy shooting");

                    // Spawn projectile
                    commands.spawn((
                        EnemyProjectile,
                        PbrBundle {
                            mesh: meshes.add(
                                shape::Box::new(ENEMY_SHOT_SIZE, ENEMY_SHOT_SIZE, ENEMY_SHOT_SIZE)
                                    .into(),
                            ),
                            material: materials.add(Color::RED.into()),
                            transform: Transform::from_xyz(
                                enemy_position.translation.x,
                                enemy_position.translation.y,
                                enemy_position.translation.z,
                            ),
                            ..default()
                        },
                    ));

                    // Reset timer
                    let duration = create_enemy_shot_timer();
                    enemy.timer = Some(Timer::from_seconds(duration, TimerMode::Once));
                }
            }
            None => {
                println!("[PROJECTILE] no timer, creating one");
                let duration = create_enemy_shot_timer();
                enemy.timer = Some(Timer::from_seconds(duration, TimerMode::Once));
            }
        }
    }
}

fn enemy_projectile_animation(mut projectiles: Query<&mut Transform, With<EnemyProjectile>>) {
    for mut projectile in projectiles.iter_mut() {
        projectile.translation.y += 0.1;
    }
}

fn detect_enemy_collision(
    mut command: Commands,
    projectiles: Query<(Entity, &Transform), With<EnemyProjectile>>,
    keys: Query<(&Transform, &PianoKeyType), With<PianoKey>>,
) {
    // Quickly check the height of piano keys
    // Get the first key
    let key_result = keys
        .iter()
        .enumerate()
        .find(|(index, _)| *index == (0 as usize));
    if let Some((_, (single_key_check, _))) = key_result {
        println!("[PROJECTILE] Found a piano key to compare");
        let key_height = single_key_check.translation.y;

        // Loop through all the projectiles and check collisions
        for (projectile_entity, projectile_position) in projectiles.iter() {
            if projectile_position.translation.y > key_height {
                println!("[PROJECTILE] Collided with player's piano");

                // Figure out which white key got hit
                let mut white_key_index = 0;
                for (key_position, key_type) in keys.iter() {
                    match key_type {
                        // White key? Check if the projectile is in piano key "lane"
                        PianoKeyType::White => {
                            let key_size = key_position.translation.x + WHITE_KEY_WIDTH;
                            if projectile_position.translation.x > key_position.translation.x
                                && projectile_position.translation.x < key_size
                            {
                                // Found the key!
                                println!("[PROJECTILE] Damage to key {}", &white_key_index);

                                // Send "damage" event to piano key

                                // Despawn / destruct projectile
                                command.entity(projectile_entity).despawn();

                                return;
                            }

                            white_key_index += 1;
                        }
                        // Ignore black keys
                        PianoKeyType::Black => {
                            return;
                        }
                    }
                }
            }
        }
    }
}

fn enemy_cleanup() {
    println!("[ENEMY] Cleaning up...");
}

fn create_enemy_shot_timer() -> f32 {
    let mut rng = rand::thread_rng();
    let duration = rng.gen_range(ENEMY_SHOOT_TIMER_MIN..ENEMY_SHOOT_TIMER_MAX);
    duration
}
