use bevy::prelude::*;
use heron::prelude::*;
use iyes_loopless::prelude::*;

use crate::{
    AppState,
    health::PlayerHealth,
    player::Player,
};

pub struct EnemiesPlugin;

impl Plugin for EnemiesPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system(follow_player_ai.run_in_state(AppState::InGame));
    }
}

#[derive(Component)]
pub struct AiFollowPlayer;

#[derive(Bundle)]
pub struct BasicEnemyBundle {
    #[bundle]
    sprite: SpriteSheetBundle,
    health: PlayerHealth,
    rigid_body: RigidBody,
    ai: AiFollowPlayer,
    velocity: Velocity,
    // TODO: Child entities.
    // hit_box,
    // hurt_box,
}

impl BasicEnemyBundle {
    pub fn new(pos: Vec2, texture_atlas: Handle<TextureAtlas>, sprite_index: usize) -> Self {
        Self {
            sprite: SpriteSheetBundle {
                sprite: TextureAtlasSprite {
                    index: sprite_index,
                    ..default()
                },
                texture_atlas,
                transform: Transform::from_translation(pos.extend(8.0)),
                ..default()
            },
            health: PlayerHealth::new(2),
            rigid_body: RigidBody::Dynamic,
            ai: AiFollowPlayer,
            velocity: default(),
        }
    }
}

fn follow_player_ai(
    player_q: Query<&Transform, With<Player>>,
    mut ai_q: Query<(&mut Velocity, &Transform), With<AiFollowPlayer>>,
) {
    if let Ok(player_transform) = player_q.get_single() {
        for (mut velocity, transform) in ai_q.iter_mut() {
            let dir = player_transform.translation.truncate() - transform.translation.truncate();
            let dir = dir.normalize_or_zero();
            let speed = 50.0;
            velocity.linear = (dir * speed).extend(0.0);
        }
    }
}
