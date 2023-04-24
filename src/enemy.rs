use std::time::Duration;

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::states::AppState;
// Resources

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
pub struct EnemyColliderEvent(pub Entity);

// Plugin

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<EnemyColliderEvent>()
            .add_system(spawn_enemies.in_schedule(OnEnter(AppState::Game)))
            // Game loop
            .add_system(mark_enemy_for_destruction.in_set(OnUpdate(AppState::Game)))
            .add_system(enemy_destruction_animation.in_set(OnUpdate(AppState::Game)))
            // Cleanup
            .add_system(enemy_cleanup.in_schedule(OnExit(AppState::Game)));
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
            enemy_data.timer = Some(Timer::from_seconds(2.0, TimerMode::Once));
        }
    }
}

pub fn enemy_destruction_animation(
    mut commands: Commands,
    mut enemies: Query<(&mut Enemy, &mut Transform, Entity)>,
    time: Res<Time>,
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
            }
        }
    }
}

fn enemy_cleanup() {
    println!("[ENEMY] Cleaning up...");
}
