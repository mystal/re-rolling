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
    game::Facing,
    health::Health,
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
            .add_system(update_player_sprite.run_in_state(AppState::InGame).after("move_player"));
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
    health: Health,
    knockback: Knockback,
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
            health: Health::new(4),
            knockback: default(),
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

#[derive(Default, Component, Reflect)]
#[reflect(Component)]
struct PlayerInput {
    movement: Vec2,
    shoot: bool,
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
    let mut shoot = false;

    // Read input from gamepad.
    if let Some(&gamepad) = gamepads.iter().next() {
        let move_x = GamepadAxis(gamepad, GamepadAxisType::LeftStickX);
        let move_y = GamepadAxis(gamepad, GamepadAxisType::LeftStickY);
        if let (Some(x), Some(y)) = (axes.get(move_x), axes.get(move_y)) {
            let tmp = Vec2::new(x, y);
            // TODO: See if we can configure the deadzone using Bevy's APIs.
            if tmp.length() > 0.1 {
                movement = tmp;
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
