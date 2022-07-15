use bevy::prelude::*;
use heron::prelude::*;
use iyes_loopless::prelude::*;

use crate::{
    GAME_LOGIC_FRAME_TIME, AppState,
    health::Health,
    physics::CollisionLayer,
};

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<HitEvent>()
            .add_system(check_hits.run_in_state(AppState::InGame).label("check_hits"))
            .add_system(deal_hit_damage.run_in_state(AppState::InGame).after("check_hits"))
            .add_system(apply_hit_knockback.run_in_state(AppState::InGame).after("check_hits"))
            .add_system(update_knockback.run_in_state(AppState::InGame).label("update_knockback"));
    }
}

#[derive(Clone, Copy)]
pub enum KnockbackDirection {
    AwayFromAttacker,
    // TowardAttacker,
    // AttackerFacing,
}

impl KnockbackDirection {
    fn compute_direction(&self, atk_pos: Vec2, def_pos: Vec2) -> Vec2 {
        match self {
            Self::AwayFromAttacker => {
                let diff = def_pos - atk_pos;
                diff.normalize_or_zero()
            }
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
    damage: u8,
    knockback: Option<KnockbackSpec>,
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
    pub fn new(size: Vec2, offset: Vec2, damage: u8, knockback: Option<KnockbackSpec>, extra_layers: &[CollisionLayer]) -> Self {
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

struct HitEvent {
    attacker: Entity,
    defender: Entity,
    damage: u8,
    knockback: Option<KnockbackSpec>,
}

fn check_hits(
    mut collisions: EventReader<CollisionEvent>,
    mut hits: EventWriter<HitEvent>,
    hit_box_q: Query<&HitBox>,
    hurt_box_q: Query<(), With<HurtBox>>,
) {
    // Listen for collision events involving a hit box and a hurt box and send a hit event.
    for collision in collisions.iter() {
        if let CollisionEvent::Started(cd1, cd2) = collision {
            let e1 = cd1.collision_shape_entity();
            let e2 = cd2.collision_shape_entity();

            // TODO: Dedup this code.
            if let Ok(hit_box) = hit_box_q.get(e1) {
                if hurt_box_q.contains(e2) {
                    hits.send(HitEvent {
                        attacker: cd1.rigid_body_entity(),
                        defender: cd2.rigid_body_entity(),
                        damage: hit_box.damage,
                        knockback: hit_box.knockback.clone(),
                    });
                }
            }
            if let Ok(hit_box) = hit_box_q.get(e2) {
                if hurt_box_q.contains(e1) {
                    hits.send(HitEvent {
                        attacker: cd2.rigid_body_entity(),
                        defender: cd1.rigid_body_entity(),
                        damage: hit_box.damage,
                        knockback: hit_box.knockback.clone(),
                    });
                }
            }
        }
    }
}

fn deal_hit_damage(
    mut hits: EventReader<HitEvent>,
    mut health_q: Query<&mut Health>,
) {
    for hit in hits.iter() {
        if let Ok(mut health) = health_q.get_mut(hit.defender) {
            health.lose_health(hit.damage);
        }
    }
}

fn apply_hit_knockback(
    mut hits: EventReader<HitEvent>,
    mut knockback_q: Query<&mut Knockback>,
    transform_q: Query<&GlobalTransform>,
) {
    for hit in hits.iter() {
        if let Some(spec) = &hit.knockback {
            if let Ok([atk_transform, def_transform]) = transform_q.get_many([hit.attacker, hit.defender]) {
                if let Ok(mut knockback) = knockback_q.get_mut(hit.defender) {
                    let (atk_pos, def_pos) = (atk_transform.translation.truncate(), def_transform.translation.truncate());
                    let direction = spec.direction.compute_direction(atk_pos, def_pos);
                    let offset = direction * spec.distance;
                    knockback.start(spec.frames, offset);
                }
            }
        }
    }
}

#[derive(Default, Component)]
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
