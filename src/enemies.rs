use bevy::prelude::*;
use heron::prelude::*;
use iyes_loopless::prelude::*;

use crate::{
    AppState,
    assets::GameAssets,
    combat::{HurtBoxBundle, Knockback},
    game::Facing,
    health::EnemyHealth,
    physics::{ColliderBundle, CollisionLayer},
    player::Player,
};

pub struct EnemiesPlugin;

impl Plugin for EnemiesPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system(follow_player_ai.run_in_state(AppState::InGame))
            .add_system_to_stage(CoreStage::PostUpdate, enemy_death.run_in_state(AppState::InGame));
    }
}

#[derive(Component)]
pub struct Enemy;

#[derive(Component)]
pub struct AiFollowPlayer;

pub fn spawn_basic_enemy(
    pos: Vec2,
    commands: &mut Commands,
    assets: &GameAssets,
) -> Entity {
    let groups = [CollisionLayer::Enemy];
    let masks = [CollisionLayer::World, CollisionLayer::Enemy];
    let collider = ColliderBundle::new(Vec2::new(13.0, 11.0), Vec2::ZERO, &groups, &masks);
    let collider = commands.spawn_bundle(collider).id();

    let groups = [CollisionLayer::Hit];
    let masks = [CollisionLayer::Player];
    let hit_box = ColliderBundle::new(Vec2::new(11.0, 9.0), Vec2::ZERO, &groups, &masks);
    let hit_box = commands.spawn_bundle(hit_box).id();

    let hurt_box = HurtBoxBundle::new(Vec2::new(13.0, 11.0), Vec2::ZERO, &[CollisionLayer::Enemy]);
    let hurt_box = commands.spawn_bundle(hurt_box).id();

    let enemy_bundle = BasicEnemyBundle::new(pos, assets.enemy_atlas.clone(), assets.enemy_indices.rat);
    commands.spawn_bundle(enemy_bundle)
        .add_child(collider)
        .add_child(hit_box)
        .add_child(hurt_box)
        .id()
}

#[derive(Bundle)]
pub struct BasicEnemyBundle {
    enemy: Enemy,
    name: Name,
    #[bundle]
    sprite: SpriteSheetBundle,
    facing: Facing,
    health: EnemyHealth,
    knockback: Knockback,
    ai: AiFollowPlayer,

    rigid_body: RigidBody,
    rotation_constraints: RotationConstraints,
    velocity: Velocity,
}

impl BasicEnemyBundle {
    pub fn new(pos: Vec2, texture_atlas: Handle<TextureAtlas>, sprite_index: usize) -> Self {
        Self {
            enemy: Enemy,
            name: Name::new("BasicEnemy"),
            sprite: SpriteSheetBundle {
                sprite: TextureAtlasSprite {
                    index: sprite_index,
                    ..default()
                },
                texture_atlas,
                transform: Transform::from_translation(pos.extend(8.0)),
                ..default()
            },
            facing: Facing { dir: Vec2::X },
            health: EnemyHealth::new(10.0),
            knockback: default(),
            ai: AiFollowPlayer,
            rigid_body: RigidBody::Dynamic,
            rotation_constraints: RotationConstraints::lock(),
            velocity: default(),
        }
    }
}

fn follow_player_ai(
    player_q: Query<&Transform, With<Player>>,
    mut ai_q: Query<(&mut Velocity, &mut Facing, &Transform, &Knockback), With<AiFollowPlayer>>,
) {
    if let Ok(player_transform) = player_q.get_single() {
        for (mut velocity, mut facing, transform, knockback) in ai_q.iter_mut() {
            if knockback.is_active() {
                return;
            }

            let dir = player_transform.translation.truncate() - transform.translation.truncate();
            let dir = dir.normalize_or_zero();
            let speed = 50.0;
            velocity.linear = (dir * speed).extend(0.0);
            facing.dir = dir;
        }
    }
}

fn enemy_death(
    mut commands: Commands,
    q: Query<(Entity, &EnemyHealth), (With<Enemy>, Changed<EnemyHealth>)>,
) {
    // TODO: Death events??
    for (entity, health) in q.iter() {
        if health.current <= 0.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}
