use bevy::prelude::*;
use bevy::math::Mat2;
use bevy_inspector_egui::{Inspectable, InspectorPlugin};
use iyes_loopless::prelude::*;

use crate::{
    AppState,
    assets::GameAssets,
    enemies,
    player::Player,
};

/// How far from the player should enemies spawn.
const SPAWN_DISTANCE: f32 = 300.0;

pub struct SpawnerPlugin;

impl Plugin for SpawnerPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<EnemyCount>()
            .add_plugin(InspectorPlugin::<EnemyCount>::new())
            .add_system(spawn_enemies.run_in_state(AppState::InGame))
            .add_system_to_stage(CoreStage::Last, update_enemy_count.run_in_state(AppState::InGame).before("despawn_dead_enemies"));
    }
}

#[derive(Default, Inspectable)]
pub struct EnemyCount(pub u32);

#[derive(Component)]
pub struct Spawner {
    pub max_enemies: u32,
    // Enemies per second.
    pub spawn_rate: f32,
    pub cooldown: f32,
}

impl Spawner {
    pub fn new(max_enemies: u32, spawn_rate: f32) -> Self {
        Self {
            max_enemies,
            spawn_rate,
            cooldown: 0.0,
        }
    }
}

fn spawn_enemies(
    mut commands: Commands,
    assets: Res<GameAssets>,
    time: Res<Time>,
    enemy_count: Res<EnemyCount>,
    mut spawner_q: Query<&mut Spawner>,
    player_q: Query<&Transform, With<Player>>,
) {
    let dt = time.delta_seconds();
    if let Ok(mut spawner) = spawner_q.get_single_mut() {
        spawner.cooldown = (spawner.cooldown - dt).max(0.0);
        if (enemy_count.0 < spawner.max_enemies) && spawner.cooldown == 0.0 {
            trace!("Spawning a basic enemy!");

            // TODO: Handle case where player doesn't exist.
            let player_pos = player_q.single().translation.truncate();
            // Pick a position randomly on the radius of a circle SPAWN_DISTANCE from the player.
            let angle = fastrand::f32() * std::f32::consts::TAU;
            let rot_matrix = Mat2::from_angle(angle);
            let offset = rot_matrix * Vec2::X * SPAWN_DISTANCE;
            let pos = player_pos + offset;
            enemies::spawn_basic_enemy(pos, &mut commands, &assets);

            spawner.cooldown = spawner.spawn_rate;
        }
    }
}

fn update_enemy_count(
    mut enemy_count: ResMut<EnemyCount>,
    spawned_q: Query<(), Added<enemies::Enemy>>,
    dead_q: Query<(), Added<enemies::Death>>,
) {
    enemy_count.0 -= dead_q.iter().count() as u32;
    enemy_count.0 += spawned_q.iter().count() as u32;
}
