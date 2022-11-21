use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use iyes_loopless::prelude::*;

use crate::{
    GAME_LOGIC_FRAME_TIME, AppState,
    enemies::Enemy,
    game::{Facing, GameTimers},
    health::{EnemyHealth, PlayerHealth},
    physics::groups,
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
    collider: Collider,
    layers: CollisionGroups,
    sensor: Sensor,
}

impl HitBoxBundle {
    // TODO: Make with_offset, with_damage, with_knockback, and with_layers methods.
    pub fn new(size: Vec2, offset: Vec2, damage: f32, knockback: Option<KnockbackSpec>, extra_layers: Group) -> Self {
        let half_extents = size / 2.0;
        let memberships = groups::HIT | extra_layers;
        let filters = groups::HURT;
        Self {
            hit_box: HitBox {
                damage,
                knockback,
            },
            transform: TransformBundle {
                local: Transform::from_translation(offset.extend(0.0)),
                ..default()
            },
            collider: Collider::cuboid(half_extents.x, half_extents.y),
            layers: CollisionGroups::new(memberships, filters),
            sensor: Sensor,
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
    collider: Collider,
    layers: CollisionGroups,
}

impl HurtBoxBundle {
    pub fn new(size: Vec2, offset: Vec2, extra_layers: Group) -> Self {
        let half_extents = size / 2.0;
        let memberships = groups::HURT | extra_layers;
        let filters = groups::HIT;
        Self {
            hurt_box: HurtBox,
            transform: TransformBundle {
                local: Transform::from_translation(offset.extend(0.0)),
                ..default()
            },
            collider: Collider::cuboid(half_extents.x, half_extents.y),
            layers: CollisionGroups::new(memberships, filters),
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

fn get_rigid_body_entity(
    entity: Entity,
    rigid_body_q: &Query<&RigidBody>,
    parent_q: &Query<&Parent>,
) -> Option<Entity> {
    if rigid_body_q.contains(entity) {
        return Some(entity);
    }

    if let Ok(parent) = parent_q.get(entity) {
        if rigid_body_q.contains(parent.get()) {
            return Some(parent.get());
        }
    }

    None
}

fn check_hits(
    mut collisions: EventReader<CollisionEvent>,
    mut hits: EventWriter<HitEvent>,
    mut player_hits: EventWriter<PlayerHitEvent>,
    parent_q: Query<&Parent>,
    rigid_body_q: Query<&RigidBody>,
    hit_box_q: Query<&HitBox>,
    hurt_box_q: Query<(), With<HurtBox>>,
    player_q: Query<(Entity, &PlayerHealth), With<Player>>,
    enemy_q: Query<(), With<Enemy>>,
    name_q: Query<&Name>,
) {
    let (player_entity, health) = player_q.single();

    // Listen for collision events involving a hit box and a hurt box and send a hit event.
    for collision in collisions.iter() {
        if let &CollisionEvent::Started(e1, e2, _flags) = collision {
            // Get parent rigid body entities.
            // TODO: Get rigid body entity. Check collider entity, if not found check parent entity.
            let rbe1 = get_rigid_body_entity(e1, &rigid_body_q, &parent_q);
            if rbe1.is_none() {
                let name = name_q.get(e1).map(|name| name.as_str()).unwrap_or("None");
                warn!("Collision happened with collider with no rigid body. Name: {}", name);
                continue;
            }
            let rbe1 = rbe1.unwrap();
            let rbe2 = get_rigid_body_entity(e2, &rigid_body_q, &parent_q);
            if rbe2.is_none() {
                let name = name_q.get(e2).map(|name| name.as_str()).unwrap_or("None");
                warn!("Collision happened with collider with no rigid body. Name: {}", name);
                continue;
            }
            let rbe2 = rbe2.unwrap();

            // TODO: Dedup this code.
            if let Ok(hit_box) = hit_box_q.get(e1) {
                if hurt_box_q.contains(e2) {
                    trace!("Hit event!");
                    hits.send(HitEvent {
                        attacker: rbe1,
                        defender: rbe2,
                        damage: hit_box.damage,
                        knockback: hit_box.knockback.clone(),
                    });
                }
            } else if let Ok(hit_box) = hit_box_q.get(e2) {
                if hurt_box_q.contains(e1) {
                    trace!("Hit event!");
                    hits.send(HitEvent {
                        attacker: rbe2,
                        defender: rbe1,
                        damage: hit_box.damage,
                        knockback: hit_box.knockback.clone(),
                    });
                }
            } else if player_entity == rbe1 && health.current > 0 && enemy_q.contains(rbe2) {
                trace!("Player hit event!");
                player_hits.send(PlayerHitEvent {
                    enemy: rbe2,
                });
            } else if player_entity == rbe2 && health.current > 0 && enemy_q.contains(rbe1) {
                trace!("Player hit event!");
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
    mut game_timers: ResMut<GameTimers>,
    mut hits: EventReader<PlayerHitEvent>,
    mut health_q: Query<&mut PlayerHealth>,
) {
    // TODO: Only allow taking damage from one source in a frame.
    let took_damage = hits.iter().count() > 0;
    if took_damage {
        if let Ok(mut health) = health_q.get_single_mut() {
            health.lose_health(1);

            if health.current == 0 {
                // Player just died!
                game_timers.game_time.pause();
                game_timers.reset_time.unpause();
            }
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
                    let (atk_pos, def_pos) = (atk_transform.translation().truncate(), def_transform.translation().truncate());
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
                let (atk_pos, def_pos) = (atk_transform.translation().truncate(), def_transform.translation().truncate());
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
            velocity.linvel = knockback.velocity;
        }

        // Tick knockback.
        knockback.seconds_remaining -= dt;
        knockback.seconds_remaining = knockback.seconds_remaining.max(0.0);
    }
}
