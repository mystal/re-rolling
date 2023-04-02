use std::ops::RangeInclusive;

use bevy::prelude::*;
use bevy::math::Mat2;
use bevy_kira_audio::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    AppState,
    animation::{self, Animation, AnimationState},
    assets::{AudioAssets, GameAssets},
    combat::*,
    game::{Facing, Lifetime},
    health::PlayerHealth,
    physics::groups,
    player::{update_player_aim, Player, PlayerInput},
};

pub struct WeaponPlugin;

impl Plugin for WeaponPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<WeaponChoice>()
            .register_type::<Weapon>()
            .add_systems((
                fire_weapon.after(update_player_aim),
                update_projectile_movement,
                boomerang_movement,
                despawn_projectile_on_hit.after(check_hits),
                explode_grenade.after(check_hits),
            ).in_set(OnUpdate(AppState::InGame)));
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Default, Reflect)]
pub enum WeaponChoice {
    #[default]
    Pistol,
    RayGun,
    Shotgun,
    Boomerang,
    Smg,
    GrenadeLauncher,
}

impl WeaponChoice {
    pub fn random() -> Self {
        let choice = fastrand::u8(0..6);
        match choice {
            0 => Self::Pistol,
            1 => Self::RayGun,
            2 => Self::Shotgun,
            3 => Self::Boomerang,
            4 => Self::Smg,
            5 => Self::GrenadeLauncher,
            _ => Self::Pistol,
        }
    }

    pub fn get_weapon_stats(&self) -> WeaponStats {
        match self {
            Self::Pistol => WeaponStats {
                max_ammo: 16,
                fire_rate: 0.3,
                projectiles_per_shot: 1,
                spread: 0.0,
            },
            Self::RayGun => WeaponStats {
                max_ammo: 12,
                fire_rate: 0.5,
                projectiles_per_shot: 1,
                spread: 0.0,
            },
            Self::Shotgun => WeaponStats {
                max_ammo: 8,
                fire_rate: 0.8,
                projectiles_per_shot: 10,
                spread: 75.0,
            },
            Self::Boomerang => WeaponStats {
                max_ammo: 8,
                fire_rate: 1.0,
                projectiles_per_shot: 1,
                spread: 0.0,
            },
            Self::Smg => WeaponStats {
                max_ammo: 64,
                fire_rate: 0.1,
                projectiles_per_shot: 1,
                spread: 30.0,
            },
            Self::GrenadeLauncher => WeaponStats {
                max_ammo: 5,
                fire_rate: 1.0,
                projectiles_per_shot: 1,
                spread: 0.0,
            },
        }
    }
}

#[derive(Default, Reflect)]
pub struct WeaponStats {
    pub max_ammo: u8,
    /// Time between each shot.
    pub fire_rate: f32,
    pub projectiles_per_shot: u8,
    // Angle in degrees of cone of spread.
    pub spread: f32,
}

#[derive(Default, Component, Reflect)]
pub struct Weapon {
    pub equipped: WeaponChoice,
    pub stats: WeaponStats,
    pub reloading: bool,
    pub ammo: u8,
    /// How long until next shot is allowed.
    pub cooldown: f32,
}

impl Weapon {
    pub fn new(choice: WeaponChoice) -> Self {
        let stats = choice.get_weapon_stats();
        Self {
            equipped: choice,
            reloading: false,
            ammo: stats.max_ammo,
            stats,
            cooldown: 0.0,
        }
    }
}

enum ProjectileSpeed {
    Single(f32),
    RandomRange(RangeInclusive<f32>),
}

#[derive(Component)]
struct DieOnHit;

#[derive(Component)]
struct ProjectileMovement {
    velocity: Vec2,
}

impl ProjectileMovement {
    fn new(velocity: Vec2) -> Self {
        Self {
            velocity,
        }
    }
}

#[derive(Bundle)]
struct ProjectileBundle {
    movement: ProjectileMovement,
    facing: Facing,
    // TODO: sprite
    #[bundle]
    sprite: SpriteSheetBundle,

    body: RigidBody,
}

impl ProjectileBundle {
    fn new(speed: f32, pos: Vec2, dir: Vec2, texture_atlas: Handle<TextureAtlas>, sprite_index: usize) -> Self {
        Self {
            movement: ProjectileMovement::new(speed * dir),
            facing: Facing { dir },
            sprite: SpriteSheetBundle {
                sprite: TextureAtlasSprite {
                    index: sprite_index,
                    ..default()
                },
                texture_atlas,
                transform: Transform::from_translation(pos.extend(15.0))
                    .with_rotation(Quat::from_rotation_z(Vec2::Y.angle_between(dir))),
                ..default()
            },
            body: RigidBody::KinematicPositionBased,
        }
    }
}

#[derive(Component)]
pub struct Grenade {
    explode_timer: f32,
    exploded: bool,
}

#[derive(Bundle)]
struct GrenadeBundle {
    grenade: Grenade,
    movement: ProjectileMovement,
    #[bundle]
    sprite: SpriteSheetBundle,

    body: RigidBody,
    velocity: Velocity,
}

impl GrenadeBundle {
    fn new(pos: Vec2, dir: Vec2, texture_atlas: Handle<TextureAtlas>, sprite_index: usize) -> Self {
        let speed = 150.0;
        let velocity = Velocity {
            linvel: dir * speed,
            ..default()
        };
        Self {
            grenade: Grenade {
                explode_timer: 0.6,
                exploded: false,
            },
            movement: ProjectileMovement::new(dir * speed),// { speed, die_on_hit: false },
            sprite: SpriteSheetBundle {
                sprite: TextureAtlasSprite {
                    index: sprite_index,
                    custom_size: Some(Vec2::new(12.0, 12.0)),
                    ..default()
                },
                texture_atlas,
                transform: Transform::from_translation(pos.extend(15.0))
                    .with_rotation(Quat::from_rotation_z(Vec2::Y.angle_between(dir))),
                ..default()
            },
            body: RigidBody::KinematicPositionBased,
            velocity,
        }
    }
}

#[derive(Bundle)]
struct ExplosionBundle {
    #[bundle]
    sprite: SpriteSheetBundle,
    name: Name,
    hit_box: HitSpec,

    body: RigidBody,
    collider: Collider,
    layers: CollisionGroups,
    sensor: Sensor,
    active_events: ActiveEvents,
    lifetime: Lifetime,
}

impl ExplosionBundle {
    fn new(pos: Vec2, texture_atlas: Handle<TextureAtlas>, sprite_index: usize) -> Self {
        Self {
            sprite: SpriteSheetBundle {
                sprite: TextureAtlasSprite {
                    index: sprite_index,
                    custom_size: Some(Vec2::new(96.0, 96.0)),
                    ..default()
                },
                texture_atlas,
                transform: Transform::from_translation(pos.extend(15.0)),
                ..default()
            },
            name: Name::new("Grenade Explosion"),
            hit_box: HitSpec::new(40.0),
            body: RigidBody::KinematicPositionBased,
            collider: Collider::ball(50.0),
            layers: CollisionGroups::new(groups::HIT, groups::HURT),
            sensor: Sensor,
            active_events: ActiveEvents::COLLISION_EVENTS,
            lifetime: Lifetime::new(0.8),
        }
    }
}

fn explode_grenade(
    mut commands: Commands,
    time: Res<Time>,
    assets: Res<GameAssets>,
    sounds: Res<AudioAssets>,
    audio: Res<Audio>,
    mut hits: EventReader<HitEvent>,
    mut grenade_q: Query<(Entity, &mut Grenade, &GlobalTransform)>,
) {
    let dt = time.delta_seconds();

    // Explode grenades either on hit or after time expires.
    for hit in hits.iter() {
        if let Ok((entity, mut grenade, transform)) = grenade_q.get_mut(hit.attacker) {
            if !grenade.exploded {
                grenade.exploded = true;
                commands.entity(entity).despawn();

                let explosion = ExplosionBundle::new(transform.translation().truncate(), assets.effects_atlas.clone(), 3);
                commands.spawn(explosion);

                audio.play(sounds.grenade_explosion.clone());
            }
        }
    }

    for (entity, mut grenade, transform) in grenade_q.iter_mut() {
        grenade.explode_timer = (grenade.explode_timer - dt).max(0.0);

        if !grenade.exploded && grenade.explode_timer == 0.0 {
            grenade.exploded = true;
            commands.entity(entity).despawn();

            let explosion = ExplosionBundle::new(transform.translation().truncate(), assets.effects_atlas.clone(), 3);
            commands.spawn(explosion);

            audio.play(sounds.grenade_explosion.clone());
        }
    }
}

#[derive(Component)]
pub struct Boomerang {
    outgoing_velocity: Vec2,
    return_time: f32,
    audio_instance: Handle<AudioInstance>,
}

#[derive(Bundle)]
struct BoomerangBundle {
    boomerang: Boomerang,
    facing: Facing,
    #[bundle]
    sprite: SpriteSheetBundle,
    anim: Handle<Animation>,
    anim_state: AnimationState,
    play: animation::Play,

    body: RigidBody,
}

impl BoomerangBundle {
    fn new(pos: Vec2, dir: Vec2, texture_atlas: Handle<TextureAtlas>, anim: Handle<Animation>, audio_instance: Handle<AudioInstance>) -> Self {
        let speed = 150.0;
        Self {
            boomerang: Boomerang {
                outgoing_velocity: dir * speed,
                return_time: 0.8,
                audio_instance,
            },
            facing: Facing { dir },
            sprite: SpriteSheetBundle {
                sprite: TextureAtlasSprite {
                    ..default()
                },
                texture_atlas,
                transform: Transform::from_translation(pos.extend(15.0))
                    .with_rotation(Quat::from_rotation_z(Vec2::Y.angle_between(dir))),
                ..default()
            },
            anim,
            anim_state: AnimationState::default(),
            play: animation::Play,
            body: RigidBody::KinematicPositionBased,
        }
    }
}

fn boomerang_movement(
    mut commands: Commands,
    time: Res<Time>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
    mut boomerang_q: Query<(Entity, &mut Transform, &mut Boomerang)>,
    player_q: Query<&GlobalTransform, With<Player>>,
) {
    // Go out in dir at first. Then return to player.
    let dt = time.delta_seconds();
    let player_transform = player_q.single();
    for (entity, mut transform, mut boomerang) in boomerang_q.iter_mut() {
        boomerang.return_time = (boomerang.return_time - dt).max(0.0);

        if boomerang.return_time > 0.0 {
            // Boomerang flying outward.
            transform.translation += (boomerang.outgoing_velocity * dt).extend(0.0);
        } else if transform.translation.distance(player_transform.translation()) < 20.0 {
            if let Some(instance) = audio_instances.get_mut(&boomerang.audio_instance) {
                instance.stop(AudioTween::default());
            }

            // Boomerang returned to player, so despawn.
            commands.entity(entity).despawn_recursive();
        } else {
            // Boomerang coming back.
            let dir = (player_transform.translation() - transform.translation).normalize_or_zero();
            transform.translation += boomerang.outgoing_velocity.length() * dir * dt;
        }
    }
}

fn fire_weapon(
    mut commands: Commands,
    time: Res<Time>,
    assets: Res<GameAssets>,
    sounds: Res<AudioAssets>,
    audio: Res<Audio>,
    mut q: Query<(&mut Weapon, &PlayerInput, &Transform, &Facing, &PlayerHealth)>,
) {
    let dt = time.delta_seconds();
    for (mut weapon, input, transform, facing, health) in q.iter_mut() {
        // Update weapon cooldown.
        weapon.cooldown = (weapon.cooldown - dt).max(0.0);

        if weapon.reloading && weapon.cooldown == 0.0 {
            // Pick new weapon!
            weapon.equipped = {
                let mut choice = WeaponChoice::random();
                while choice == weapon.equipped {
                    choice = WeaponChoice::random();
                }
                choice
            };
            weapon.stats = weapon.equipped.get_weapon_stats();
            weapon.ammo = weapon.stats.max_ammo;

            weapon.reloading = false;

            // Don't shoot this frame.
            continue;
        }

        // Check if we want to shoot and can shoot.
        if !input.shoot || weapon.cooldown > 0.0 || weapon.ammo == 0 || health.current == 0 {
            continue;
        }

        // Get projectile properties.
        let (damage, knockback, sprite_index, speed, lifetime, hit_box_size, die_on_hit, name) = match weapon.equipped {
            WeaponChoice::Pistol => (
                4.0,
                KnockbackSpec {
                    direction: KnockbackDirection::AttackerFacing,
                    frames: 6,
                    distance: 10.0,
                },
                assets.projectile_indices.bullet,
                ProjectileSpeed::Single(200.0),
                2.0,
                Vec2::new(2.0, 4.0),
                true,
                "Projectile: Pistol",
            ),
            WeaponChoice::RayGun => (
                5.0,
                KnockbackSpec {
                    direction: KnockbackDirection::AttackerFacing,
                    frames: 6,
                    distance: 8.0,
                },
                assets.projectile_indices.laser,
                ProjectileSpeed::Single(200.0),
                5.0,
                Vec2::new(2.0, 4.0),
                false,
                "Projectile: RayGun",
            ),
            WeaponChoice::Shotgun => (
                8.0,
                KnockbackSpec {
                    direction: KnockbackDirection::AttackerFacing,
                    frames: 6,
                    distance: 20.0,
                },
                assets.projectile_indices.bullet,
                ProjectileSpeed::RandomRange(100.0..=200.0),
                0.5,
                Vec2::new(2.0, 4.0),
                true,
                "Projectile: Shotgun",
            ),
            WeaponChoice::Boomerang => (
                3.0,
                KnockbackSpec {
                    direction: KnockbackDirection::AwayFromAttacker,
                    frames: 12,
                    distance: 14.0,
                },
                assets.projectile_indices.bullet,
                ProjectileSpeed::Single(200.0),
                20.0,
                Vec2::new(6.0, 6.0),
                false,
                "Projectile: Boomerang",
            ),
            WeaponChoice::Smg => (
                2.0,
                KnockbackSpec {
                    direction: KnockbackDirection::AttackerFacing,
                    frames: 6,
                    distance: 6.0,
                },
                assets.projectile_indices.bullet,
                ProjectileSpeed::Single(200.0),
                2.0,
                Vec2::new(2.0, 4.0),
                true,
                "Projectile: SMG",
            ),
            WeaponChoice::GrenadeLauncher => (
                20.0,
                KnockbackSpec {
                    direction: KnockbackDirection::AttackerFacing,
                    frames: 6,
                    distance: 40.0,
                },
                assets.projectile_indices.grenade,
                ProjectileSpeed::Single(200.0),
                10.0,
                Vec2::new(4.0, 4.0),
                true,
                "Projectile: Grenade Launcher",
            ),
        };

        // Determine fire direction. Either aim direction or facing if aim is zero.
        let aim_dir = if input.aim != Vec2::ZERO {
            input.aim.normalize_or_zero()
        } else {
            facing.dir
        };

        // Spawn projectiles.
        for _ in 0..weapon.stats.projectiles_per_shot {
            // Compute common data.
            let fire_dir = if weapon.stats.spread > 0.0 {
                // Rotate dir based on spread.
                let spread_angle = (fastrand::f32() * weapon.stats.spread) - (weapon.stats.spread / 2.0);
                Mat2::from_angle(spread_angle.to_radians()) * aim_dir
            } else {
                aim_dir
            };
            let pos = transform.translation.truncate() + (fire_dir * 10.0);
            let hit_box = HitSpec::new(damage)
                .with_knockback(knockback.clone());
            let collider_shape = Collider::cuboid(hit_box_size.x, hit_box_size.y);
            let collision_layers = CollisionGroups::new(groups::HIT, groups::HURT);

            if weapon.equipped == WeaponChoice::GrenadeLauncher {
                let bundle = GrenadeBundle::new(pos, fire_dir, assets.projectile_atlas.clone(), sprite_index);
                let mut builder = commands.spawn((
                    bundle,
                    Name::new(name),
                    hit_box,
                    collider_shape,
                    collision_layers,
                    Sensor,
                    ActiveEvents::COLLISION_EVENTS,
                ));
                if die_on_hit {
                    builder.insert(DieOnHit);
                }

                audio.play(sounds.grenade.clone());
            } else if weapon.equipped == WeaponChoice::Boomerang {
                let audio_instance = audio.play(sounds.boomerang.clone()).looped().handle();

                let bundle = BoomerangBundle::new(pos, fire_dir, assets.boomerang_atlas.clone(), assets.boomerang_anim.clone(), audio_instance);
                let mut builder = commands.spawn((
                    bundle,
                    Name::new(name),
                    hit_box,
                    collider_shape,
                    collision_layers,
                    Sensor,
                    ActiveEvents::COLLISION_EVENTS,
                ));
                if die_on_hit {
                    builder.insert(DieOnHit);
                }
            } else {
                let speed = match &speed {
                    ProjectileSpeed::Single(s) => *s,
                    ProjectileSpeed::RandomRange(range) => {
                        let range_delta = range.end() - range.start();
                        range.start() + (range_delta * fastrand::f32())
                    }
                };
                let projectile_bundle = ProjectileBundle::new(speed, pos, fire_dir, assets.projectile_atlas.clone(), sprite_index);
                let mut builder = commands.spawn((
                    projectile_bundle,
                    Name::new(name),
                    Lifetime::new(lifetime),
                    hit_box,
                    collider_shape,
                    collision_layers,
                    Sensor,
                    ActiveEvents::COLLISION_EVENTS,
                ));
                if die_on_hit {
                    builder.insert(DieOnHit);
                }

                let (sound, volume) = match weapon.equipped {
                    WeaponChoice::Pistol => (&sounds.pistol, 0.4),
                    WeaponChoice::RayGun => (&sounds.raygun, 1.0),
                    WeaponChoice::Shotgun => (&sounds.shotgun, 0.5),
                    WeaponChoice::Boomerang => (&sounds.boomerang, 1.0),
                    WeaponChoice::Smg => (&sounds.smg, 0.7),
                    WeaponChoice::GrenadeLauncher => (&sounds.grenade, 1.0),
                };

                audio.play(sound.clone()).with_volume(volume);
            }
        }

        // Spend ammo and start cooldown.
        weapon.ammo -= 1;
        weapon.cooldown = if weapon.ammo != 0 {
            weapon.stats.fire_rate
        } else {
            weapon.reloading = true;
            2.0
        };
    }
}

fn update_projectile_movement(
    time: Res<Time>,
    mut q: Query<(&mut Transform, &ProjectileMovement)>,
) {
    let dt = time.delta_seconds();
    for (mut transform, movement) in q.iter_mut() {
        transform.translation += (movement.velocity * dt).extend(0.0);
    }
}

fn despawn_projectile_on_hit(
    mut commands: Commands,
    mut hits: EventReader<HitEvent>,
    die_on_hit_q: Query<(), With<DieOnHit>>,
) {
    for hit in hits.iter() {
        if die_on_hit_q.contains(hit.attacker) {
            commands.entity(hit.attacker).despawn();
        }
    }
}
