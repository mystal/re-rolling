use bevy::prelude::*;
use heron::prelude::*;
use iyes_loopless::prelude::*;

use crate::{
    GAME_LOGIC_FRAME_TIME, AppState,
    enemies::Enemy,
    game::Facing,
    health::{EnemyHealth, PlayerHealth},
    physics::CollisionLayer,
    player::Player,
};

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<Knockback>()
            .add_event::<HitEvent>()
            .add_event::<PlayerHitEvent>()
            .add_system(check_hits.run_in_state(AppState::InGame).label("check_hits"))
            .add_system(deal_hit_damage.run_in_state(AppState::InGame).label("deal_hit_damage").after("check_hits"))
            .add_system(deal_player_hit_damage.run_in_state(AppState::InGame).label("deal_player_hit_damage").after("check_hits"))
            .add_system(apply_hit_knockback.run_in_state(AppState::InGame).after("check_hits"))
            .add_system(apply_player_hit_knockback.run_in_state(AppState::InGame).after("check_hits"))
            .add_system(update_knockback.run_in_state(AppState::InGame).label("update_knockback"));
    }
}

#[derive(Clone, Copy)]
pub enum KnockbackDirection {
    AwayFromAttacker,
    // TowardAttacker,
    AttackerFacing,
}

impl KnockbackDirection {
    fn compute_direction(&self, atk_pos: Vec2, def_pos: Vec2, atk_facing: Vec2) -> Vec2 {
        match self {
            Self::AwayFromAttacker => {
                let diff = def_pos - atk_pos;
                diff.normalize_or_zero()
            }
            Self::AttackerFacing => atk_facing,
        }
    }
}

#[derive(Clone)]
pub struct KnockbackSpec {
    pub direction: KnockbackDirection,
    pub frames: u8,
    pub distance: f32,
}

#[derive(Component)]
pub struct HitBox {
    damage: f32,
    knockback: Option<KnockbackSpec>,
}

impl HitBox {
    pub fn new(mut damage: f32) -> Self {
        if damage < 0.0 {
            warn!("Tried to create a HitBox with negative damage ({}), setting to 0!", damage);
            damage = 0.0;
        }
        Self {
            damage,
            knockback: None,
        }
    }

    pub fn with_knockback(mut self, knockback: KnockbackSpec) -> Self {
        self.knockback = Some(knockback);
        self
    }
}

#[derive(Bundle)]
pub struct HitBoxBundle {
    hit_box: HitBox,
    #[bundle]
    transform: TransformBundle,
    collider: CollisionShape,
    layers: CollisionLayers,
}

impl HitBoxBundle {
    // TODO: Make with_offset, with_damage, with_knockback, and with_layers methods.
    pub fn new(size: Vec2, offset: Vec2, damage: f32, knockback: Option<KnockbackSpec>, extra_layers: &[CollisionLayer]) -> Self {
        Self {
            hit_box: HitBox {
                damage,
                knockback,
            },
            transform: TransformBundle {
                local: Transform::from_translation(offset.extend(0.0)),
                ..default()
            },
            collider: CollisionShape::Cuboid {
                half_extends: (size / 2.0).extend(0.0),
                border_radius: None,
            },
            layers: CollisionLayers::none()
                .with_group(CollisionLayer::Hit)
                .with_groups(extra_layers)
                .with_masks([CollisionLayer::Hurt]),
        }
    }
}

#[derive(Component)]
pub struct HurtBox;

#[derive(Bundle)]
pub struct HurtBoxBundle {
    hurt_box: HurtBox,
    #[bundle]
    transform: TransformBundle,
    collider: CollisionShape,
    layers: CollisionLayers,
}

impl HurtBoxBundle {
    pub fn new(size: Vec2, offset: Vec2, extra_layers: &[CollisionLayer]) -> Self {
        Self {
            hurt_box: HurtBox,
            transform: TransformBundle {
                local: Transform::from_translation(offset.extend(0.0)),
                ..default()
            },
            collider: CollisionShape::Cuboid {
                half_extends: (size / 2.0).extend(0.0),
                border_radius: None,
            },
            layers: CollisionLayers::none()
                .with_group(CollisionLayer::Hurt)
                .with_groups(extra_layers)
                .with_masks([CollisionLayer::Hit]),
        }
    }
}

pub struct HitEvent {
    pub attacker: Entity,
    pub defender: Entity,
    pub damage: f32,
    pub knockback: Option<KnockbackSpec>,
}

pub struct PlayerHitEvent {
    pub enemy: Entity,
}

fn check_hits(
    mut collisions: EventReader<CollisionEvent>,
    mut hits: EventWriter<HitEvent>,
    mut player_hits: EventWriter<PlayerHitEvent>,
    hit_box_q: Query<&HitBox>,
    hurt_box_q: Query<(), With<HurtBox>>,
    player_q: Query<(), With<Player>>,
    enemy_q: Query<(), With<Enemy>>,
) {
    // Listen for collision events involving a hit box and a hurt box and send a hit event.
    for collision in collisions.iter() {
        if let CollisionEvent::Started(cd1, cd2) = collision {
            let e1 = cd1.collision_shape_entity();
            let e2 = cd2.collision_shape_entity();
            let rbe1 = cd1.rigid_body_entity();
            let rbe2 = cd2.rigid_body_entity();

            // TODO: Dedup this code.
            if let Ok(hit_box) = hit_box_q.get(e1) {
                if hurt_box_q.contains(e2) {
                    debug!("Hit event!");
                    hits.send(HitEvent {
                        attacker: cd1.rigid_body_entity(),
                        defender: cd2.rigid_body_entity(),
                        damage: hit_box.damage,
                        knockback: hit_box.knockback.clone(),
                    });
                }
            } else if let Ok(hit_box) = hit_box_q.get(e2) {
                if hurt_box_q.contains(e1) {
                    debug!("Hit event!");
                    hits.send(HitEvent {
                        attacker: cd2.rigid_body_entity(),
                        defender: cd1.rigid_body_entity(),
                        damage: hit_box.damage,
                        knockback: hit_box.knockback.clone(),
                    });
                }
            } else if player_q.contains(rbe1) && enemy_q.contains(rbe2) {
                debug!("Player hit event!");
                player_hits.send(PlayerHitEvent {
                    enemy: rbe2,
                });
            } else if player_q.contains(rbe2) && enemy_q.contains(rbe1) {
                debug!("Player hit event!");
                player_hits.send(PlayerHitEvent {
                    enemy: rbe1,
                });
            }
        }
    }
}

fn deal_hit_damage(
    mut hits: EventReader<HitEvent>,
    mut health_q: Query<&mut EnemyHealth>,
) {
    for hit in hits.iter() {
        if let Ok(mut health) = health_q.get_mut(hit.defender) {
            health.lose_health(hit.damage);
        }
    }
}

fn deal_player_hit_damage(
    mut hits: EventReader<PlayerHitEvent>,
    mut health_q: Query<&mut PlayerHealth>,
) {
    // TODO: Only allow taking damage from one source in a frame.
    for _ in hits.iter() {
        if let Ok(mut health) = health_q.get_single_mut() {
            health.lose_health(1);
        }
    }
}

fn apply_hit_knockback(
    mut hits: EventReader<HitEvent>,
    mut knockback_q: Query<&mut Knockback>,
    transform_q: Query<(&GlobalTransform, &Facing)>,
) {
    for hit in hits.iter() {
        if let Some(spec) = &hit.knockback {
            if let Ok([(atk_transform, atk_facing), (def_transform, _)]) = transform_q.get_many([hit.attacker, hit.defender]) {
                if let Ok(mut knockback) = knockback_q.get_mut(hit.defender) {
                    let (atk_pos, def_pos) = (atk_transform.translation.truncate(), def_transform.translation.truncate());
                    let direction = spec.direction.compute_direction(atk_pos, def_pos, atk_facing.dir);
                    let offset = direction * spec.distance;
                    knockback.start(spec.frames, offset);
                }
            }
        }
    }
}

fn apply_player_hit_knockback(
    mut hits: EventReader<PlayerHitEvent>,
    mut knockback_q: Query<(Entity, &mut Knockback), With<Player>>,
    transform_q: Query<(&GlobalTransform, &Facing)>,
) {
    for hit in hits.iter() {
        if let Ok((player_entity, mut knockback)) = knockback_q.get_single_mut() {
            if let Ok([(atk_transform, atk_facing), (def_transform, _)]) = transform_q.get_many([hit.enemy, player_entity]) {
                let (atk_pos, def_pos) = (atk_transform.translation.truncate(), def_transform.translation.truncate());
                let direction = KnockbackDirection::AwayFromAttacker.compute_direction(atk_pos, def_pos, atk_facing.dir);
                let offset = direction * 25.0;
                knockback.start(10, offset);
            }
        }
    }
}

#[derive(Default, Component, Reflect)]
#[reflect(Component)]
pub struct Knockback {
    seconds_remaining: f32,
    velocity: Vec2,
}

impl Knockback {
    pub fn is_active(&self) -> bool {
        self.seconds_remaining > 0.0
    }

    pub fn start(&mut self, frames: u8, offset: Vec2) {
        self.seconds_remaining = frames as f32 * GAME_LOGIC_FRAME_TIME;
        self.velocity = offset / self.seconds_remaining;
    }
}

fn update_knockback(
    time: Res<Time>,
    mut knockback_q: Query<(&mut Knockback, Option<&mut Velocity>)>,
) {
    let dt = time.delta_seconds();
    for (mut knockback, maybe_velocity) in knockback_q.iter_mut() {
        if !knockback.is_active() {
            continue;
        }

        // Update velocity.
        if let Some(mut velocity) = maybe_velocity {
            velocity.linear = knockback.velocity.extend(0.0);
        }

        // Tick knockback.
        knockback.seconds_remaining -= dt;
        knockback.seconds_remaining = knockback.seconds_remaining.max(0.0);
    }
}
