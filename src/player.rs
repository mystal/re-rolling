use benimator::SpriteSheetAnimation;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy_egui::EguiContext;
use bevy_inspector_egui::prelude::*;
use heron::prelude::*;
use iyes_loopless::prelude::*;

use crate::{
    AppState,
    assets::GameAssets,
    combat::Knockback,
    game::{Facing, Lifetime},
    health::PlayerHealth,
    physics::CollisionLayer,
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_inspectable::<PlayerMovement>()
            .register_type::<PlayerInput>()
            .add_system(read_player_input.run_in_state(AppState::InGame).label("player_input"))
            .add_system(update_player_movement.run_in_state(AppState::InGame).label("move_player").after("player_input"))
            .add_system(update_player_sprite.run_in_state(AppState::InGame).after("move_player"))
            .add_system(update_player_aim.run_in_state(AppState::InGame).label("update_player_aim").after("player_input"))
            .add_system(fire_weapon.run_in_state(AppState::InGame).after("update_player_aim"))
            .add_system(update_projectile_movement.run_in_state(AppState::InGame));
    }
}

#[derive(Bundle)]
pub struct PlayerBundle {
    player: Player,
    // TODO: Move sprite and anim to a child entity of the player.
    #[bundle]
    sprite: SpriteSheetBundle,
    anim: Handle<SpriteSheetAnimation>,
    name: Name,
    body: RigidBody,
    rotation_constraints: RotationConstraints,
    velocity: Velocity,
    facing: Facing,
    movement: PlayerMovement,
    input: PlayerInput,
    play: benimator::Play,
    health: PlayerHealth,
    knockback: Knockback,
    weapon: Weapon,
}

impl PlayerBundle {
    pub fn new(pos: Vec2, atlas: Handle<TextureAtlas>, anim: Handle<SpriteSheetAnimation>) -> Self {
        let pos = pos.extend(10.0);
        Self {
            player: Player::default(),
            sprite: SpriteSheetBundle {
                sprite: TextureAtlasSprite {
                    anchor: Anchor::BottomCenter,
                    ..default()
                },
                texture_atlas: atlas,
                transform: Transform::from_translation(pos),
                ..default()
            },
            anim,
            name: Name::new("Player"),
            body: RigidBody::Dynamic,
            rotation_constraints: RotationConstraints::lock(),
            velocity: Velocity::default(),
            facing: default(),
            movement: PlayerMovement { walk_speed: 100.0 },
            input: default(),
            play: benimator::Play,
            health: PlayerHealth::new(4),
            knockback: default(),
            weapon: Weapon::new(8, 0.3),
        }
    }
}

#[derive(Bundle)]
pub struct PlayerColliderBundle {
    #[bundle]
    transform: TransformBundle,
    shape: CollisionShape,
    layers: CollisionLayers,
}

impl PlayerColliderBundle {
    pub fn new() -> Self {
        Self {
            transform: TransformBundle {
                local: Transform::from_translation(Vec3::new(0.0, 5.0, 0.0)),
                ..default()
            },
            shape: CollisionShape::Cuboid {
                half_extends: Vec3::new(5.0, 5.0, 0.0),
                border_radius: None,
            },
            layers: CollisionLayers::none()
                .with_groups([CollisionLayer::Player, CollisionLayer::Collision])
                .with_masks([CollisionLayer::Collision]),
        }
    }
}

#[derive(Default, Component)]
pub struct Player {
}

#[derive(Component, Inspectable)]
struct PlayerMovement {
    walk_speed: f32,
}

#[derive(Component, Inspectable)]
struct PlayerAim(pub Vec2);

#[derive(Default, Component, Reflect)]
#[reflect(Component)]
struct PlayerInput {
    movement: Vec2,
    aim: Vec2,
    shoot: bool,
}

#[derive(Component, Inspectable)]
struct Weapon {
    ammo: u8,
    max_ammo: u8,
    /// Time between each shot.
    fire_rate: f32,
    /// How long until next shot is allowed.
    cooldown: f32,
}

impl Weapon {
    fn new(max_ammo: u8, fire_rate: f32) -> Self {
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

fn read_player_input(
    keys: Res<Input<KeyCode>>,
    mouse_buttons: Res<Input<MouseButton>>,
    gamepads: Res<Gamepads>,
    pad_buttons: Res<Input<GamepadButton>>,
    axes: Res<Axis<GamepadAxis>>,
    mut egui_ctx: ResMut<EguiContext>,
    mut q: Query<&mut PlayerInput>,
) {
    let mut movement = Vec2::ZERO;
    let mut aim = Vec2::ZERO;
    let mut shoot = false;

    // Read input from gamepad.
    if let Some(&gamepad) = gamepads.iter().next() {
        // Movement
        let move_x = GamepadAxis(gamepad, GamepadAxisType::LeftStickX);
        let move_y = GamepadAxis(gamepad, GamepadAxisType::LeftStickY);
        if let (Some(x), Some(y)) = (axes.get(move_x), axes.get(move_y)) {
            let tmp = Vec2::new(x, y);
            // TODO: See if we can configure the deadzone using Bevy's APIs.
            if tmp.length() > 0.1 {
                movement = tmp;
            }
        }

        // Aim
        let aim_x = GamepadAxis(gamepad, GamepadAxisType::RightStickX);
        let aim_y = GamepadAxis(gamepad, GamepadAxisType::RightStickY);
        if let (Some(x), Some(y)) = (axes.get(aim_x), axes.get(aim_y)) {
            let tmp = Vec2::new(x, y);
            // TODO: See if we can configure the deadzone using Bevy's APIs.
            if tmp.length() > 0.1 {
                aim = tmp;
            }
        }

        let shoot_button = GamepadButton(gamepad, GamepadButtonType::RightTrigger2);
        shoot |= pad_buttons.pressed(shoot_button);
    }

    // Read input from mouse/keyboard.
    if movement == Vec2::ZERO && !egui_ctx.ctx_mut().wants_keyboard_input() {
        let x = (keys.pressed(KeyCode::D) as i8 - keys.pressed(KeyCode::A) as i8) as f32;
        let y = (keys.pressed(KeyCode::W) as i8 - keys.pressed(KeyCode::S) as i8) as f32;
        movement = Vec2::new(x, y).normalize_or_zero();
    }
    shoot |= mouse_buttons.pressed(MouseButton::Left) && !egui_ctx.ctx_mut().wants_pointer_input();

    // Store results in player input component.
    for mut input in q.iter_mut() {
        input.movement = movement;
        input.aim = aim;
        input.shoot = shoot;
    }
}

fn update_player_movement(
    mut q: Query<(&PlayerMovement, &PlayerInput, &mut Velocity, &mut Facing, &Knockback)>,
) {
    for (movement, input, mut velocity, mut facing, knockback) in q.iter_mut() {
        if knockback.is_active() {
            continue;
        }

        velocity.linear = (input.movement * movement.walk_speed).extend(0.0);

        if input.movement != Vec2::ZERO {
            facing.dir = input.movement.normalize_or_zero();
        }
    }
}

fn update_player_aim(
    mut q: Query<(&mut PlayerAim, &PlayerInput)>,
) {
    for (mut aim, input) in q.iter_mut() {
        if input.aim != Vec2::ZERO {
            aim.0 = input.aim.normalize_or_zero();
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
        let projectile_bundle = ProjectileBundle::new(200.0, pos, dir, assets.projectile_atlas.clone(), assets.projectile_indices.bullet);
        commands.spawn_bundle(projectile_bundle)
            .insert(Lifetime::new(2.0));

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

fn update_player_sprite(
    assets: Res<GameAssets>,
    mut player_q: Query<(&Facing, &PlayerInput, &Knockback, &mut TextureAtlasSprite, &mut Handle<SpriteSheetAnimation>)>,
) {
    for (facing, input, knockback, mut sprite, mut anim) in player_q.iter_mut() {
        if facing.dir.x != 0.0 {
            sprite.flip_x = facing.dir.x < 0.0;
        }

        if knockback.is_active() {
            *anim = assets.player_anims.hit_react.clone();
        } else if input.movement.length() > 0.1 {
            *anim = assets.player_anims.run.clone();
        } else {
            *anim = assets.player_anims.idle.clone();
        }
    }
}
