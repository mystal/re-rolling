use bevy::prelude::*;
use bevy::sprite::Anchor;
use heron::prelude::*;
use iyes_loopless::prelude::*;

use crate::{
    AppState,
    assets::GameAssets,
    combat,
    health::Health,
    physics::CollisionLayer,
    player,
    window::WindowScale,
};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
         app
            .add_plugin(combat::CombatPlugin)
            .add_plugin(player::PlayerPlugin)
            .register_type::<Facing>()
            .register_type::<Health>()
            .add_enter_system(AppState::InGame, setup_game)
            .add_system_to_stage(CoreStage::PostUpdate, camera_follows_player.run_in_state(AppState::InGame));
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

    let player_bundle = player::PlayerBundle::new(Vec2::ZERO, assets.player_atlas.clone(), assets.player_anims.idle.clone());
    commands.spawn_bundle(player_bundle);
}

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

fn camera_follows_player(
    player_q: Query<&Transform, (With<player::Player>, Changed<Transform>)>,
    mut camera_q: Query<&mut Transform, (With<Camera>, Without<player::Player>)>,
) {
    if let (Ok(mut camera_transform), Ok(player_transform)) = (camera_q.get_single_mut(), player_q.get_single()) {
        camera_transform.translation.x = player_transform.translation.x;
        camera_transform.translation.y = player_transform.translation.y;
    }
}
