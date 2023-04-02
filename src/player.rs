use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::window::{PrimaryWindow, WindowRef};
use bevy_egui::EguiContexts;
use bevy_rapier2d::prelude::*;

use crate::{
    AppState,
    animation::{self, Animation, AnimationState},
    assets::GameAssets,
    combat::*,
    game::{Crosshair, Facing},
    health::PlayerHealth,
    physics::{groups, ColliderBundle},
    weapons::{Weapon, WeaponChoice, WeaponPlugin},
};

const PLAYER_Z: f32 = 10.0;
const POST_HIT_INVULN: f32 = 1.0;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugin(WeaponPlugin)
            .register_type::<PlayerMovement>()
            .register_type::<PlayerInput>()
            .add_system(read_player_input.in_set(OnUpdate(AppState::InGame)))
            .add_system(update_player_movement.in_set(OnUpdate(AppState::InGame)).after(read_player_input))
            .add_system(update_player_sprite.in_set(OnUpdate(AppState::InGame)).after(update_player_movement))
            .add_system(update_player_aim.in_set(OnUpdate(AppState::InGame)).after(read_player_input))
            .add_system(update_crosshair.in_set(OnUpdate(AppState::InGame)).after(update_player_aim))
            .add_system(update_post_hit_invuln.in_set(OnUpdate(AppState::InGame)))
            .add_system(apply_post_hit_invuln.in_set(OnUpdate(AppState::InGame)).after(deal_player_hit_damage))
            .add_system(flicker_player_during_invuln.in_base_set(CoreSet::PostUpdate));
    }
}

pub fn spawn_player(
    pos: Vec2,
    commands: &mut Commands,
    assets: &GameAssets,
) -> Entity {
    let crosshair_bundle = SpriteSheetBundle {
        sprite: TextureAtlasSprite {
            color: Color::rgba(1.0, 1.0, 1.0, 0.6),
            ..default()
        },
        texture_atlas: assets.crosshair_atlas.clone(),
        visibility: Visibility::Hidden,
        ..default()
    };
    let crosshair = commands.spawn(crosshair_bundle.clone())
        .insert(Crosshair)
        .insert(Name::new("ControllerCrosshair"))
        .id();
    commands.spawn(crosshair_bundle)
        .insert(Crosshair)
        .insert(Name::new("MouseCrosshair"));

    let groups = groups::PLAYER;
    let masks = groups::WORLD;
    let collider = ColliderBundle::new(Vec2::new(11.0, 11.0), Vec2::ZERO, groups, masks);
    let collider = commands.spawn(collider)
        .insert(Name::new("PlayerCollider"))
        .insert(ActiveEvents::COLLISION_EVENTS)
        .id();

    let groups = groups::PLAYER;
    let masks = groups::HIT;
    let hurt_box = ColliderBundle::new(Vec2::new(8.0, 8.0), Vec2::ZERO, groups, masks);
    let hurt_box = commands.spawn(hurt_box)
        .insert(Name::new("PlayerHurtBox"))
        .insert(ActiveEvents::COLLISION_EVENTS)
        .id();

    let player_bundle = PlayerBundle::new(pos, assets.player_atlas.clone(), assets.player_anims.idle.clone());
    commands.spawn(player_bundle)
        .insert(Player { hurt_box })
        .add_child(crosshair)
        .add_child(collider)
        .add_child(hurt_box)
        .id()
}

#[derive(Bundle)]
pub struct PlayerBundle {
    // TODO: Move sprite and anim to a child entity of the player.
    #[bundle]
    sprite: SpriteSheetBundle,
    anim: Handle<Animation>,
    anim_state: AnimationState,
    name: Name,
    body: RigidBody,
    rotation_constraints: LockedAxes,
    velocity: Velocity,
    facing: Facing,
    movement: PlayerMovement,
    input: PlayerInput,
    play: animation::Play,
    health: PlayerHealth,
    knockback: Knockback,
    weapon: Weapon,
    post_hit_invuln: PostHitInvulnerability,
}

impl PlayerBundle {
    pub fn new(pos: Vec2, atlas: Handle<TextureAtlas>, anim: Handle<Animation>) -> Self {
        let pos = pos.extend(PLAYER_Z);
        Self {
            sprite: SpriteSheetBundle {
                sprite: TextureAtlasSprite {
                    anchor: Anchor::Custom(Vec2::new(0.0, -0.15)),
                    ..default()
                },
                texture_atlas: atlas,
                transform: Transform::from_translation(pos),
                ..default()
            },
            anim,
            anim_state: AnimationState::default(),
            name: Name::new("Player"),
            body: RigidBody::Dynamic,
            rotation_constraints: LockedAxes::ROTATION_LOCKED,
            velocity: Velocity::default(),
            facing: default(),
            movement: PlayerMovement { walk_speed: 100.0 },
            input: default(),
            play: animation::Play,
            health: PlayerHealth::new(4),
            knockback: default(),
            weapon: Weapon::new(WeaponChoice::default()),
            post_hit_invuln: default(),
        }
    }
}

#[derive(Component)]
pub struct Player {
    hurt_box: Entity,
}

#[derive(Component, Reflect)]
struct PlayerMovement {
    walk_speed: f32,
}

#[derive(Component, Reflect)]
pub struct PlayerAim(pub Vec2);

#[derive(Default, Component, Reflect)]
pub struct PlayerInput {
    pub movement: Vec2,
    pub aim: Vec2,
    pub aim_device: AimDevice,
    pub shoot: bool,
    pub next_weapon: bool,
    pub prev_weapon: bool,
    pub reset_game: bool,
}

#[derive(Default, Component)]
pub struct PostHitInvulnerability {
    remaining: f32,
}

impl PostHitInvulnerability {
    fn is_active(&self) -> bool {
        self.remaining > 0.0
    }

    fn start(&mut self) {
        self.remaining = POST_HIT_INVULN;
    }

    fn tick(&mut self, dt: f32) {
        self.remaining = (self.remaining - dt).max(0.0);
    }
}

// Taken from:
// https://bevy-cheatbook.github.io/cookbook/cursor2world.html#2d-games
fn get_mouse_world_pos(
    window_q: &Query<&Window>,
    primary_window_q: &Query<&Window, With<PrimaryWindow>>,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> Option<Vec2> {
    use bevy::render::camera::RenderTarget;

    // Get the window that the camera is displaying to (or the primary window).
    let window = match camera.target {
        RenderTarget::Window(WindowRef::Entity(entity)) => window_q.get(entity).unwrap(),
        _ => primary_window_q.single(),
    };

    // Check if the cursor is inside the window and get its position.
    if let Some(screen_pos) = window.cursor_position() {
        // Get the size of the window.
        let window_size = Vec2::new(window.width() as f32, window.height() as f32);

        // Convert screen position [0..resolution] to ndc [-1..1] (gpu coordinates).
        let ndc = (screen_pos / window_size) * 2.0 - Vec2::ONE;

        // Matrix for undoing the projection and camera transform.
        let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix().inverse();

        // Use it to convert ndc to world-space coordinates.
        let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));

        // Reduce it to a 2D value.
        Some(world_pos.truncate())
    } else {
        None
    }
}

#[derive(Clone, Copy, Default, Reflect)]
pub enum AimDevice {
    #[default]
    None,
    Gamepad,
    Mouse(Vec2),
}

pub fn read_player_input(
    keys: Res<Input<KeyCode>>,
    mouse_buttons: Res<Input<MouseButton>>,
    mut cursor_moved: EventReader<CursorMoved>,
    gamepads: Res<Gamepads>,
    pad_buttons: Res<Input<GamepadButton>>,
    axes: Res<Axis<GamepadAxis>>,
    mut egui_ctx: EguiContexts,
    mut player_q: Query<(&mut PlayerInput, &GlobalTransform)>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    window_q: Query<&Window>,
    primary_window_q: Query<&Window, With<PrimaryWindow>>,
) {
    let (mut input, player_transform) = player_q.single_mut();

    let mut movement = Vec2::ZERO;
    let mut aim = Vec2::ZERO;
    let mut aim_device = input.aim_device;
    let mut shoot = false;
    let mut reset_game = false;

    // Read input from gamepad.
    if let Some(gamepad) = gamepads.iter().next() {
        // Movement
        let move_x = GamepadAxis::new(gamepad, GamepadAxisType::LeftStickX);
        let move_y = GamepadAxis::new(gamepad, GamepadAxisType::LeftStickY);
        if let (Some(x), Some(y)) = (axes.get(move_x), axes.get(move_y)) {
            let tmp = Vec2::new(x, y);
            // TODO: See if we can configure the deadzone using Bevy's APIs.
            if tmp.length() > 0.1 {
                movement = tmp;
            }
        }

        // Aim
        let aim_x = GamepadAxis::new(gamepad, GamepadAxisType::RightStickX);
        let aim_y = GamepadAxis::new(gamepad, GamepadAxisType::RightStickY);
        if let (Some(x), Some(y)) = (axes.get(aim_x), axes.get(aim_y)) {
            let tmp = Vec2::new(x, y);
            // TODO: See if we can configure the deadzone using Bevy's APIs.
            if tmp.length() > 0.1 {
                aim = tmp;
                aim_device = AimDevice::Gamepad;
            } else {
                aim_device = AimDevice::None;
            }
        }

        // Shoot
        let shoot_button = GamepadButton::new(gamepad, GamepadButtonType::RightTrigger2);
        shoot |= pad_buttons.pressed(shoot_button);

        let reset_button = GamepadButton::new(gamepad, GamepadButtonType::Start);
        reset_game |= pad_buttons.pressed(reset_button);
    }

    // Read input from mouse/keyboard.
    // Movement
    if movement == Vec2::ZERO && !egui_ctx.ctx_mut().wants_keyboard_input() {
        let x = (keys.pressed(KeyCode::D) as i8 - keys.pressed(KeyCode::A) as i8) as f32;
        let y = (keys.pressed(KeyCode::W) as i8 - keys.pressed(KeyCode::S) as i8) as f32;
        movement = Vec2::new(x, y).normalize_or_zero();
    }

    // Aim
    let mouse_moved = cursor_moved.iter().count() > 0;
    // Try to use mouse for aim if the gamepad isn't being used and the mouse moved or we were
    // already using the mouse.
    if aim == Vec2::ZERO && (mouse_moved || matches!(input.aim_device, AimDevice::Mouse(_))) {
        let (camera, camera_transform) = camera_q.single();
        if let Some(pos) = get_mouse_world_pos(&window_q, &primary_window_q, camera, camera_transform) {
            aim = (pos - player_transform.translation().truncate()).normalize_or_zero();
            aim_device = AimDevice::Mouse(pos);
        }
    }

    // Shoot
    shoot |= mouse_buttons.pressed(MouseButton::Left) && !egui_ctx.ctx_mut().wants_pointer_input();

    reset_game |= keys.just_pressed(KeyCode::Space) && !egui_ctx.ctx_mut().wants_keyboard_input();

    // Store results in player input component.
    input.movement = movement;
    input.aim = aim;
    input.aim_device = aim_device;
    input.shoot = shoot;
    input.reset_game = reset_game;
}

fn update_player_movement(
    mut q: Query<(&PlayerMovement, &PlayerInput, &mut Velocity, &mut Facing, &Knockback, &PlayerHealth)>,
) {
    for (movement, input, mut velocity, mut facing, knockback, health) in q.iter_mut() {
        if knockback.is_active() {
            continue;
        }

        if health.current == 0 {
            velocity.linvel = Vec2::ZERO;
        } else {
            velocity.linvel = input.movement * movement.walk_speed;
        }

        if input.movement != Vec2::ZERO {
            facing.dir = input.movement.normalize_or_zero();
        }
    }
}

pub fn update_player_aim(
    mut q: Query<(&mut PlayerAim, &PlayerInput)>,
) {
    for (mut aim, input) in q.iter_mut() {
        if input.aim != Vec2::ZERO {
            aim.0 = input.aim.normalize_or_zero();
        }
    }
}

fn update_player_sprite(
    assets: Res<GameAssets>,
    mut player_q: Query<(&PlayerHealth, &PlayerInput, &Knockback, &mut Handle<Animation>)>,
) {
    for (health, input, knockback, mut anim) in player_q.iter_mut() {
        if health.current == 0 {
            *anim = assets.player_anims.dead.clone();
        } else if knockback.is_active() {
            *anim = assets.player_anims.hit_react.clone();
        } else if input.movement.length() > 0.1 {
            *anim = assets.player_anims.run.clone();
        } else {
            *anim = assets.player_anims.idle.clone();
        }
    }
}

fn update_crosshair(
    mut gamepad_crosshair_q: Query<(&mut Transform, &mut Visibility), (With<Crosshair>, With<Parent>)>,
    mut mouse_crosshair_q: Query<(&mut Transform, &mut Visibility), (With<Crosshair>, Without<Parent>)>,
    input_q: Query<&PlayerInput>,
) {
    if let Ok(input) = input_q.get_single() {
        if let Ok((mut transform, mut visibility)) = gamepad_crosshair_q.get_single_mut() {
            // TODO: Smooth jittery movement. Maybe don't hide it when not explicitly aiming?
            if let AimDevice::Gamepad = input.aim_device {
                let dir = input.aim.normalize_or_zero();
                let offset = 50.0;
                transform.translation = (dir * offset).extend(1.0);
                *visibility = Visibility::Inherited;
            } else {
                *visibility = Visibility::Hidden;
            }
        }

        if let Ok((mut transform, mut visibility)) = mouse_crosshair_q.get_single_mut() {
            if let AimDevice::Mouse(pos) = input.aim_device {
                transform.translation = pos.extend(PLAYER_Z + 1.0);
                *visibility = Visibility::Inherited;
            } else {
                *visibility = Visibility::Hidden;
            }
        }
    }
}

fn apply_post_hit_invuln(
    mut player_q: Query<(&Player, &mut PostHitInvulnerability, &PlayerHealth), Changed<PlayerHealth>>,
    mut hurt_box_q: Query<&mut CollisionGroups>,
) {
    for (player, mut invuln, health) in player_q.iter_mut() {
        if health.current > 0 && health.missing() > 0 {
            invuln.start();

            // Invulnerability started, clear hurt box collision layers.
            if let Ok(mut layers) = hurt_box_q.get_mut(player.hurt_box) {
                *layers = CollisionGroups::new(Group::NONE, Group::NONE);
            }
        }
    }
}

fn update_post_hit_invuln(
    time: Res<Time>,
    mut player_q: Query<(&Player, &mut PostHitInvulnerability)>,
    mut hurt_box_q: Query<&mut CollisionGroups>,
) {
    let dt = time.delta_seconds();
    for (player, mut invuln) in player_q.iter_mut() {
        if invuln.is_active() {
            invuln.tick(dt);
            if !invuln.is_active() {
                // Invulnerability ended, reset hurt box collision layers.
                if let Ok(mut layers) = hurt_box_q.get_mut(player.hurt_box) {
                    *layers = CollisionGroups::new(groups::PLAYER, groups::HIT);
                }
            }
        }
    }
}

fn flicker_player_during_invuln(
    time: Res<Time>,
    mut q: Query<(&PostHitInvulnerability, &mut Visibility)>,
) {
    // Flicker ten times a second.
    let just_millis = time.elapsed().as_millis() % 1000;
    let bucket = just_millis / 100;
    let visible = if bucket % 2 == 0 { Visibility::Inherited } else { Visibility::Hidden };
    for (invuln, mut visibility) in q.iter_mut() {
        if invuln.is_active() {
            *visibility = visible;
        } else {
            *visibility = Visibility::Inherited;
        }
    }
}
