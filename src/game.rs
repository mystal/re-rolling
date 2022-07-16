use bevy::prelude::*;
use iyes_loopless::prelude::*;

use crate::{
    AppState,
    assets::GameAssets,
    combat,
    enemies,
    health::PlayerHealth,
    player,
    window::WindowScale,
};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
         app
            .add_plugin(combat::CombatPlugin)
            .add_plugin(enemies::EnemiesPlugin)
            .add_plugin(player::PlayerPlugin)
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
) {
    let mut camera_bundle = OrthographicCameraBundle::new_2d();
    camera_bundle.orthographic_projection.scale = 1.0 / window_scale.0 as f32;
    commands.spawn_bundle(camera_bundle);

    let crosshair_bundle = SpriteSheetBundle {
        sprite: TextureAtlasSprite {
            color: Color::rgba(0.9, 0.9, 0.9, 0.9),
            ..default()
        },
        texture_atlas: assets.crosshair_atlas.clone(),
        visibility: Visibility {
            is_visible: false,
        },
        ..default()
    };
    let crosshair = commands.spawn_bundle(crosshair_bundle)
        .insert(Crosshair)
        .id();

    let player_bundle = player::PlayerBundle::new(Vec2::ZERO, assets.player_atlas.clone(), assets.player_anims.idle.clone());
    commands.spawn_bundle(player_bundle)
        .add_child(crosshair);

    let enemy_bundle = enemies::BasicEnemyBundle::new(Vec2::new(300.0, 0.0), assets.enemy_atlas.clone(), assets.enemy_indices.rat);
    commands.spawn_bundle(enemy_bundle);
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
