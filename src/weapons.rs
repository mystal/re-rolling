use bevy::prelude::*;
use heron::prelude::*;
use iyes_loopless::prelude::*;

use crate::{
    AppState,
    assets::GameAssets,
    combat::*,
    game::{Facing, Lifetime},
    physics::CollisionLayer,
    player::PlayerInput,
};

pub struct WeaponPlugin;

impl Plugin for WeaponPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system(fire_weapon.run_in_state(AppState::InGame).after("update_player_aim"))
            .add_system(update_projectile_movement.run_in_state(AppState::InGame))
            .add_system(despawn_projectile_on_hit.run_in_state(AppState::InGame).after("check_hits"));
    }
}

#[derive(Default, Component, Reflect)]
#[reflect(Component)]
pub struct Weapon {
    pub ammo: u8,
    pub max_ammo: u8,
    /// Time between each shot.
    pub fire_rate: f32,
    /// How long until next shot is allowed.
    pub cooldown: f32,
}

impl Weapon {
    pub fn new(max_ammo: u8, fire_rate: f32) -> Self {
        Self {
            ammo: max_ammo,
            max_ammo,
            fire_rate,
            cooldown: 0.0,
        }
    }
}

#[derive(Component)]
struct ProjectileMovement {
    speed: f32,
}

impl ProjectileMovement {
    fn new(speed: f32) -> Self {
        Self {
            speed,
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
    name: Name,

    body: RigidBody,
    velocity: Velocity,
}

impl ProjectileBundle {
    fn new(speed: f32, pos: Vec2, dir: Vec2, texture_atlas: Handle<TextureAtlas>, sprite_index: usize) -> Self {
        let velocity = Velocity::from_linear((dir * speed).extend(0.0));
        Self {
            movement: ProjectileMovement::new(speed),
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

fn fire_weapon(
    mut commands: Commands,
    time: Res<Time>,
    assets: Res<GameAssets>,
    mut q: Query<(&mut Weapon, &PlayerInput, &Transform, &Facing)>,
) {
    let dt = time.delta_seconds();
    for (mut weapon, input, transform, facing) in q.iter_mut() {
        // Update weapon cooldown.
        weapon.cooldown = (weapon.cooldown - dt).max(0.0);

        // Check if we want to shoot and can shoot.
        if !input.shoot || weapon.cooldown > 0.0 {
            continue;
        }

        // TODO: Take ammo into account and add reload!

        // Spawn projectile.
        // Shoot either in direction aim is pointing or facing if aim is zero.
        let dir = if input.aim != Vec2::ZERO {
            input.aim.normalize_or_zero()
        } else {
            facing.dir
        };
        let pos = transform.translation.truncate() + (dir * 10.0);
        let hit_box = HitBox::new(3.0)
            .with_knockback(KnockbackSpec {
                direction: KnockbackDirection::AttackerFacing,
                frames: 6,
                distance: 10.0,
            });
        let collider_shape = CollisionShape::Cuboid {
            half_extends: Vec3::new(2.0, 4.0, 0.0),
            border_radius: None,
        };
        let collision_layers = CollisionLayers::none()
            .with_groups([CollisionLayer::Hit])
            .with_masks([CollisionLayer::Hurt]);
        let projectile_bundle = ProjectileBundle::new(200.0, pos, dir, assets.projectile_atlas.clone(), assets.projectile_indices.bullet);
        commands.spawn_bundle(projectile_bundle)
            .insert(Lifetime::new(2.0))
            .insert(hit_box)
            .insert(collider_shape)
            .insert(collision_layers);

        weapon.cooldown = weapon.fire_rate;
    }
}

fn update_projectile_movement(
    time: Res<Time>,
    mut q: Query<(&mut Transform, &Velocity), With<ProjectileMovement>>,
) {
    let dt = time.delta_seconds();
    for (mut transform, velocity) in q.iter_mut() {
        transform.translation += velocity.linear * dt;
    }
}

fn despawn_projectile_on_hit(
    mut commands: Commands,
    mut hits: EventReader<HitEvent>,
    projectile_q: Query<(), With<ProjectileMovement>>,
) {
    for hit in hits.iter() {
        if projectile_q.contains(hit.attacker) {
            commands.entity(hit.attacker).despawn();
        }
    }
}
