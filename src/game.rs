use bevy::prelude::*;
use iyes_loopless::prelude::*;

use crate::{
    AppState,
    assets::GameAssets,
    combat,
    enemies,
    health::PlayerHealth,
    player,
    terrain,
    ui,
    window::WindowScale,
};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
         app
            .add_plugin(combat::CombatPlugin)
            .add_plugin(enemies::EnemiesPlugin)
            .add_plugin(player::PlayerPlugin)
            .add_plugin(terrain::TerrainPlugin)
            .add_plugin(ui::UiPlugin)
            .register_type::<Facing>()
            .register_type::<PlayerHealth>()
            .add_enter_system(AppState::InGame, setup_game)
            .add_system_to_stage(CoreStage::PostUpdate, camera_follows_player.run_in_state(AppState::InGame))
            .add_system_to_stage(CoreStage::PostUpdate, update_lifetimes.run_in_state(AppState::InGame));
    }
}

fn setup_game(
    mut commands: Commands,
    assets: Res<GameAssets>,
    window_scale: Res<WindowScale>,
    mut spawned_chunks: ResMut<terrain::SpawnedChunks>,
) {
    let mut camera_bundle = OrthographicCameraBundle::new_2d();
    camera_bundle.orthographic_projection.scale = 1.0 / window_scale.0 as f32;
    commands.spawn_bundle(camera_bundle);

    player::spawn_player(Vec2::ZERO, &mut commands, &assets);

    enemies::spawn_basic_enemy(Vec2::new(300.0, 0.0), &mut commands, &assets);

    commands.spawn()
        .insert(enemies::spawner::Spawner::new(50, 1.0));

    // Spawn initial terrain chunks.
    terrain::spawn_missing_chunks(IVec2::ZERO, &mut commands, &assets, &mut spawned_chunks);
}

#[derive(Component)]
pub struct Crosshair;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Facing {
    pub dir: Vec2,
}

impl Default for Facing {
    fn default() -> Self {
        Self {
            dir: Vec2::X,
        }
    }
}

#[derive(Component)]
pub struct Lifetime {
    pub lifetime: f32,
    pub remaining: f32,
}

impl Lifetime {
    pub fn new(seconds: f32) -> Self {
        Self {
            lifetime: seconds,
            remaining: seconds,
        }
    }
}

fn update_lifetimes(
    mut commands: Commands,
    time: Res<Time>,
    mut q: Query<(Entity, &mut Lifetime)>,
) {
    let dt = time.delta_seconds();
    for (entity, mut lifetime) in q.iter_mut() {
        lifetime.remaining = (lifetime.remaining - dt).max(0.0);
        if lifetime.remaining <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn camera_follows_player(
    player_q: Query<&Transform, (With<player::Player>, Changed<Transform>)>,
    mut camera_q: Query<&mut Transform, (With<Camera>, Without<player::Player>)>,
) {
    if let (Ok(mut camera_transform), Ok(player_transform)) = (camera_q.get_single_mut(), player_q.get_single()) {
        camera_transform.translation.x = player_transform.translation.x;
        camera_transform.translation.y = player_transform.translation.y;
    }
}
