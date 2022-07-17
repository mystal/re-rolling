use benimator::SpriteSheetAnimation;
use bevy::prelude::*;
use bevy::math::Mat2;
use bevy_inspector_egui::{Inspectable, RegisterInspectable};
use heron::prelude::*;
use iyes_loopless::prelude::*;

use crate::{
    AppState,
    assets::GameAssets,
    combat::*,
    game::{Facing, Lifetime},
    health::PlayerHealth,
    physics::CollisionLayer,
    player::{Player, PlayerInput},
};

pub struct WeaponPlugin;

impl Plugin for WeaponPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_inspectable::<WeaponChoice>()
            .register_inspectable::<Weapon>()
            .add_system(fire_weapon.run_in_state(AppState::InGame).after("update_player_aim"))
            .add_system(update_projectile_movement.run_in_state(AppState::InGame))
            .add_system(boomerang_movement.run_in_state(AppState::InGame))
            .add_system(despawn_projectile_on_hit.run_in_state(AppState::InGame).after("check_hits"))
            .add_system(explode_grenade.run_in_state(AppState::InGame).after("check_hits"));
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Default, Inspectable)]
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
                max_ammo: 10,
                fire_rate: 0.6,
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

#[derive(Default, Inspectable)]
pub struct WeaponStats {
    pub max_ammo: u8,
    /// Time between each shot.
    pub fire_rate: f32,
    pub projectiles_per_shot: u8,
    // Angle in degrees of cone of spread.
    pub spread: f32,
}

#[derive(Default, Component, Inspectable)]
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

#[derive(Component)]
struct Projectile {
    speed: f32,
    die_on_hit: bool,
}

impl Projectile {
    fn new(speed: f32, die_on_hit: bool) -> Self {
        Self {
            speed,
            die_on_hit,
        }
    }
}

#[derive(Bundle)]
struct ProjectileBundle {
    movement: Projectile,
    facing: Facing,
    // TODO: sprite
    #[bundle]
    sprite: SpriteSheetBundle,
    name: Name,

    body: RigidBody,
    velocity: Velocity,
}

impl ProjectileBundle {
    fn new(speed: f32, die_on_hit: bool, pos: Vec2, dir: Vec2, texture_atlas: Handle<TextureAtlas>, sprite_index: usize) -> Self {
        let velocity = Velocity::from_linear((dir * speed).extend(0.0));
        Self {
            movement: Projectile::new(speed, die_on_hit),
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
            name: Name::new("Projectile"),
            body: RigidBody::Sensor,
            velocity,
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
    projectile: Projectile,
    #[bundle]
    sprite: SpriteSheetBundle,
    name: Name,

    body: RigidBody,
    velocity: Velocity,
}

impl GrenadeBundle {
    fn new(pos: Vec2, dir: Vec2, texture_atlas: Handle<TextureAtlas>, sprite_index: usize) -> Self {
        let speed = 150.0;
        let velocity = Velocity::from_linear((dir * speed).extend(0.0));
        Self {
            grenade: Grenade {
                explode_timer: 0.6,
                exploded: false,
            },
            projectile: Projectile { speed, die_on_hit: false },
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
            name: Name::new("Grenade"),
            body: RigidBody::Sensor,
            velocity,
        }
    }
}

#[derive(Bundle)]
struct ExplosionBundle {
    #[bundle]
    sprite: SpriteSheetBundle,
    name: Name,
    hit_box: HitBox,

    body: RigidBody,
    collider: CollisionShape,
    layers: CollisionLayers,
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
            name: Name::new("Grenade"),
            hit_box: HitBox::new(40.0),
            body: RigidBody::Sensor,
            collider: CollisionShape::Sphere { radius: 50.0 },
            layers: CollisionLayers::none()
                .with_group(CollisionLayer::Hit)
                .with_mask(CollisionLayer::Hurt),
            lifetime: Lifetime::new(0.8),
        }
    }
}

fn explode_grenade(
    mut commands: Commands,
    time: Res<Time>,
    assets: Res<GameAssets>,
    mut hits: EventReader<HitEvent>,
    mut grenade_q: Query<(Entity, &mut Grenade, &GlobalTransform)>,
) {
    let dt = time.delta_seconds();

    // TODO: Either on hit or after time expires.
    for hit in hits.iter() {
        if let Ok((entity, mut grenade, transform)) = grenade_q.get_mut(hit.attacker) {
            if !grenade.exploded {
                grenade.exploded = true;
                commands.entity(entity).despawn();

                let explosion = ExplosionBundle::new(transform.translation.truncate(), assets.effects_atlas.clone(), 3);
                commands.spawn_bundle(explosion);
            }
        }
    }

    for (entity, mut grenade, transform) in grenade_q.iter_mut() {
        grenade.explode_timer = (grenade.explode_timer - dt).max(0.0);

        if !grenade.exploded && grenade.explode_timer == 0.0 {
            grenade.exploded = true;
            commands.entity(entity).despawn();

            let explosion = ExplosionBundle::new(transform.translation.truncate(), assets.effects_atlas.clone(), 3);
            commands.spawn_bundle(explosion);
        }
    }
}

#[derive(Component)]
pub struct Boomerang {
    return_time: f32
}

#[derive(Bundle)]
struct BoomerangBundle {
    boomerang: Boomerang,
    facing: Facing,
    #[bundle]
    sprite: SpriteSheetBundle,
    anim: Handle<SpriteSheetAnimation>,
    play: benimator::Play,
    name: Name,

    body: RigidBody,
    velocity: Velocity,
}

impl BoomerangBundle {
    fn new(pos: Vec2, dir: Vec2, texture_atlas: Handle<TextureAtlas>, anim: Handle<SpriteSheetAnimation>) -> Self {
        let speed = 150.0;
        let velocity = Velocity::from_linear((dir * speed).extend(0.0));
        Self {
            boomerang: Boomerang {
                return_time: 0.8,
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
            play: benimator::Play,
            name: Name::new("Boomerang"),
            body: RigidBody::Sensor,
            velocity,
        }
    }
}

fn boomerang_movement(
    mut commands: Commands,
    time: Res<Time>,
    mut boomerang_q: Query<(Entity, &mut Transform, &mut Boomerang, &Velocity)>,
    player_q: Query<&GlobalTransform, With<Player>>,
) {
    // TODO: Go out in dir at first. Then return to player.
    let dt = time.delta_seconds();
    let player_transform = player_q.single();
    for (entity, mut transform, mut boomerang, velocity) in boomerang_q.iter_mut() {
        boomerang.return_time = (boomerang.return_time - dt).max(0.0);

        if boomerang.return_time > 0.0 {
            transform.translation += velocity.linear * dt;
        } else if transform.translation.distance(player_transform.translation) < 20.0 {
            commands.entity(entity).despawn_recursive();
        } else {
            let dir = (player_transform.translation - transform.translation).normalize_or_zero();
            transform.translation += velocity.linear.length() * dir * dt;
        }
    }
}

fn fire_weapon(
    mut commands: Commands,
    time: Res<Time>,
    assets: Res<GameAssets>,
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
        let (damage, knockback, sprite_index, speed, lifetime, hit_box_size, die_on_hit) = match weapon.equipped {
            WeaponChoice::Pistol => (
                3.0,
                10.0,
                assets.projectile_indices.bullet,
                200.0,
                2.0,
                Vec2::new(2.0, 4.0),
                true,
            ),
            WeaponChoice::RayGun => (
                4.0,
                6.0,
                assets.projectile_indices.laser,
                200.0,
                5.0,
                Vec2::new(2.0, 4.0),
                false,
            ),
            WeaponChoice::Shotgun => (
                8.0,
                20.0,
                assets.projectile_indices.bullet,
                150.0,
                0.5,
                Vec2::new(2.0, 4.0),
                true,
            ),
            WeaponChoice::Boomerang => (
                2.0,
                14.0,
                assets.projectile_indices.bullet,
                200.0,
                20.0,
                Vec2::new(6.0, 6.0),
                false,
            ),
            WeaponChoice::Smg => (
                2.0,
                6.0,
                assets.projectile_indices.bullet,
                200.0,
                2.0,
                Vec2::new(2.0, 4.0),
                true,
            ),
            WeaponChoice::GrenadeLauncher => (
                20.0,
                40.0,
                assets.projectile_indices.grenade,
                200.0,
                10.0,
                Vec2::new(4.0, 4.0),
                true,
            ),
        };

        // Spawn projectiles.
        for _ in 0..weapon.stats.projectiles_per_shot {
            if weapon.equipped == WeaponChoice::GrenadeLauncher {
                let dir = {
                    // Shoot either in direction aim is pointing or facing if aim is zero.
                    let mut dir = if input.aim != Vec2::ZERO {
                        input.aim.normalize_or_zero()
                    } else {
                        facing.dir
                    };
                    // Rotate dir based on spread.
                    if weapon.stats.spread > 0.0 {
                        let spread_angle = (fastrand::f32() * weapon.stats.spread) - (weapon.stats.spread / 2.0);
                        dir = Mat2::from_angle(spread_angle.to_radians()) * dir;
                    }
                    dir
                };
                let pos = transform.translation.truncate() + (dir * 10.0);
                let hit_box = HitBox::new(damage)
                    .with_knockback(KnockbackSpec {
                        direction: KnockbackDirection::AttackerFacing,
                        frames: 6,
                        distance: knockback,
                    });
                let collider_shape = CollisionShape::Cuboid {
                    half_extends: hit_box_size.extend(0.0),
                    border_radius: None,
                };
                let collision_layers = CollisionLayers::none()
                    .with_groups([CollisionLayer::Hit])
                    .with_masks([CollisionLayer::Hurt]);
                let bundle = GrenadeBundle::new(pos, dir, assets.projectile_atlas.clone(), sprite_index);
                commands.spawn_bundle(bundle)
                    .insert(hit_box)
                    .insert(collider_shape)
                    .insert(collision_layers);
            } else if weapon.equipped == WeaponChoice::Boomerang {
                let dir = {
                    // Shoot either in direction aim is pointing or facing if aim is zero.
                    let mut dir = if input.aim != Vec2::ZERO {
                        input.aim.normalize_or_zero()
                    } else {
                        facing.dir
                    };
                    // Rotate dir based on spread.
                    if weapon.stats.spread > 0.0 {
                        let spread_angle = (fastrand::f32() * weapon.stats.spread) - (weapon.stats.spread / 2.0);
                        dir = Mat2::from_angle(spread_angle.to_radians()) * dir;
                    }
                    dir
                };
                let pos = transform.translation.truncate() + (dir * 10.0);
                let hit_box = HitBox::new(damage)
                    .with_knockback(KnockbackSpec {
                        direction: KnockbackDirection::AwayFromAttacker,
                        frames: 12,
                        distance: knockback,
                    });
                let collider_shape = CollisionShape::Cuboid {
                    half_extends: hit_box_size.extend(0.0),
                    border_radius: None,
                };
                let collision_layers = CollisionLayers::none()
                    .with_groups([CollisionLayer::Hit])
                    .with_masks([CollisionLayer::Hurt]);
                let bundle = BoomerangBundle::new(pos, dir, assets.boomerang_atlas.clone(), assets.boomerang_anim.clone());
                commands.spawn_bundle(bundle)
                    .insert(hit_box)
                    .insert(collider_shape)
                    .insert(collision_layers);
            } else {
                let dir = {
                    // Shoot either in direction aim is pointing or facing if aim is zero.
                    let mut dir = if input.aim != Vec2::ZERO {
                        input.aim.normalize_or_zero()
                    } else {
                        facing.dir
                    };
                    // Rotate dir based on spread.
                    if weapon.stats.spread > 0.0 {
                        let spread_angle = (fastrand::f32() * weapon.stats.spread) - (weapon.stats.spread / 2.0);
                        dir = Mat2::from_angle(spread_angle.to_radians()) * dir;
                    }
                    dir
                };
                let pos = transform.translation.truncate() + (dir * 10.0);
                let hit_box = HitBox::new(damage)
                    .with_knockback(KnockbackSpec {
                        direction: KnockbackDirection::AttackerFacing,
                        frames: 6,
                        distance: knockback,
                    });
                let collider_shape = CollisionShape::Cuboid {
                    half_extends: hit_box_size.extend(0.0),
                    border_radius: None,
                };
                let collision_layers = CollisionLayers::none()
                    .with_groups([CollisionLayer::Hit])
                    .with_masks([CollisionLayer::Hurt]);
                let projectile_bundle = ProjectileBundle::new(speed, die_on_hit, pos, dir, assets.projectile_atlas.clone(), sprite_index);
                commands.spawn_bundle(projectile_bundle)
                    .insert(Lifetime::new(lifetime))
                    .insert(hit_box)
                    .insert(collider_shape)
                    .insert(collision_layers);
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
    mut q: Query<(&mut Transform, &Velocity), With<Projectile>>,
) {
    let dt = time.delta_seconds();
    for (mut transform, velocity) in q.iter_mut() {
        transform.translation += velocity.linear * dt;
    }
}

fn despawn_projectile_on_hit(
    mut commands: Commands,
    mut hits: EventReader<HitEvent>,
    projectile_q: Query<&Projectile>,
) {
    for hit in hits.iter() {
        if let Ok(projectile) = projectile_q.get(hit.attacker) {
            if projectile.die_on_hit {
                commands.entity(hit.attacker).despawn();
            }
        }
    }
}
