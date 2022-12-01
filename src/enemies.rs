use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use iyes_loopless::prelude::*;

use crate::{
    AppState,
    animation::{Animation, AnimationState, Play},
    assets::GameAssets,
    combat::{HurtBoxBundle, Knockback},
    game::{Facing, Lifetime},
    health::EnemyHealth,
    physics::{groups, ColliderBundle},
    player::Player,
};

pub mod spawner;

pub struct EnemiesPlugin;

impl Plugin for EnemiesPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugin(spawner::SpawnerPlugin)
            .add_system(follow_player_ai.run_in_state(AppState::InGame))
            .add_system_to_stage(CoreStage::PostUpdate, trigger_enemy_death.run_in_state(AppState::InGame))
            .add_system_to_stage(CoreStage::Last, despawn_dead_enemies.run_in_state(AppState::InGame).label("despawn_dead_enemies"));
    }
}

#[derive(Component)]
pub struct Enemy;

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Death;

#[derive(Component)]
pub struct AiFollowPlayer;

pub fn spawn_basic_enemy(
    pos: Vec2,
    commands: &mut Commands,
    assets: &GameAssets,
) -> Entity {
    let groups = groups::ENEMY;
    let masks = groups::WORLD | groups::ENEMY;
    let collider = ColliderBundle::new(Vec2::new(13.0, 11.0), Vec2::ZERO, groups, masks);
    let collider = commands.spawn(collider)
        .insert(Name::new("EnemyCollider"))
        .id();

    let groups = groups::HIT;
    let masks = groups::PLAYER;
    let hit_box = ColliderBundle::new(Vec2::new(11.0, 9.0), Vec2::ZERO, groups, masks);
    let hit_box = commands.spawn(hit_box)
        .insert(Name::new("EnemyHitBox"))
        .id();

    let hurt_box = HurtBoxBundle::new(Vec2::new(13.0, 11.0), Vec2::ZERO, groups::ENEMY);
    let hurt_box = commands.spawn(hurt_box)
        .insert(Name::new("EnemyHurtBox"))
        .id();

    let enemy_bundle = BasicEnemyBundle::new(pos, assets.enemy_atlas.clone(), assets.enemy_indices.rat);
    commands.spawn(enemy_bundle)
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
    rotation_constraints: LockedAxes,
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
            rotation_constraints: LockedAxes::ROTATION_LOCKED,
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
            velocity.linvel = dir * speed;
            facing.dir = dir;
        }
    }
}

fn trigger_enemy_death(
    mut commands: Commands,
    q: Query<(Entity, &EnemyHealth), (With<Enemy>, Changed<EnemyHealth>)>,
) {
    // TODO: Death events??
    for (entity, health) in q.iter() {
        if health.current <= 0.0 {
            commands.entity(entity)
                .insert(Death);
        }
    }
}

fn despawn_dead_enemies(
    mut commands: Commands,
    assets: Res<GameAssets>,
    q: Query<(Entity, &GlobalTransform), (With<Enemy>, Added<Death>)>,
) {
    for (entity, transform) in q.iter() {
        commands.entity(entity).despawn_recursive();

        // Spawn VFX entity.
        // TODO: Make this configurable in the future.
        let vfx_bundle = VfxBundle::new(
            transform.translation(),
            assets.explosions_atlas.clone(),
            assets.explosion_anim.clone(),
        );

        // TODO: Don't spawn VFX bundle if enemies killed when restarting.
        commands.spawn(vfx_bundle);
    }
}

#[derive(Bundle)]
struct VfxBundle {
    name: Name,
    sprite: SpriteSheetBundle,
    anim: Handle<Animation>,
    anim_state: AnimationState,
    play: Play,
    lifetime: Lifetime,
}

impl VfxBundle {
    fn new(pos: Vec3, texture_atlas: Handle<TextureAtlas>, anim: Handle<Animation>) -> Self {
        Self {
            name: "EnemyDeathVfx".into(),
            sprite: SpriteSheetBundle {
                texture_atlas,
                transform: Transform::from_translation(pos),
                ..default()
            },
            anim,
            anim_state: AnimationState::default(),
            play: Play,
            lifetime: Lifetime::new(0.4),
        }
    }
}
