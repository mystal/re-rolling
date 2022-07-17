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
    combat::*,
    game::{Crosshair, Facing},
    health::PlayerHealth,
    physics::{ColliderBundle, CollisionLayer},
    weapons::{Weapon, WeaponChoice, WeaponPlugin},
};

const PLAYER_Z: f32 = 10.0;
const POST_HIT_INVULN: f32 = 1.0;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugin(WeaponPlugin)
            .register_inspectable::<PlayerMovement>()
            .register_inspectable::<PlayerInput>()
            .add_system(read_player_input.run_in_state(AppState::InGame).label("player_input"))
            .add_system(update_player_movement.run_in_state(AppState::InGame).label("move_player").after("player_input"))
            .add_system(update_player_sprite.run_in_state(AppState::InGame).after("move_player"))
            .add_system(update_player_aim.run_in_state(AppState::InGame).label("update_player_aim").after("player_input"))
            .add_system(update_crosshair.run_in_state(AppState::InGame).after("update_player_aim"))
            .add_system(update_post_hit_invuln.run_in_state(AppState::InGame))
            .add_system(apply_post_hit_invuln.run_in_state(AppState::InGame).after("deal_player_hit_damage"))
            .add_system_to_stage(CoreStage::PostUpdate, flicker_player_during_invuln);
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
        visibility: Visibility {
            is_visible: false,
        },
        ..default()
    };
    let crosshair = commands.spawn_bundle(crosshair_bundle.clone())
        .insert(Crosshair)
        .id();
    commands.spawn_bundle(crosshair_bundle)
        .insert(Crosshair);

    let groups = [CollisionLayer::Player];
    let masks = [CollisionLayer::World];
    let collider = ColliderBundle::new(Vec2::new(11.0, 11.0), Vec2::ZERO, &groups, &masks);
    let collider = commands.spawn_bundle(collider).id();

    let groups = [CollisionLayer::Player];
    let masks = [CollisionLayer::Hit];
    let hurt_box = ColliderBundle::new(Vec2::new(8.0, 8.0), Vec2::ZERO, &groups, &masks);
    let hurt_box = commands.spawn_bundle(hurt_box).id();

    let player_bundle = PlayerBundle::new(pos, assets.player_atlas.clone(), assets.player_anims.idle.clone());
    commands.spawn_bundle(player_bundle)
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
    post_hit_invuln: PostHitInvulnerability,
}

impl PlayerBundle {
    pub fn new(pos: Vec2, atlas: Handle<TextureAtlas>, anim: Handle<SpriteSheetAnimation>) -> Self {
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
            weapon: Weapon::new(WeaponChoice::Shotgun),
            post_hit_invuln: default(),
        }
    }
}

#[derive(Component)]
pub struct Player {
    hurt_box: Entity,
}

#[derive(Component, Inspectable)]
struct PlayerMovement {
    walk_speed: f32,
}

#[derive(Component, Inspectable)]
struct PlayerAim(pub Vec2);

#[derive(Default, Component, Inspectable)]
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
    wnds: &Windows,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> Option<Vec2> {
    use bevy::render::camera::RenderTarget;

    // Get the window that the camera is displaying to (or the primary window).
    let wnd = if let RenderTarget::Window(id) = camera.target {
        wnds.get(id).unwrap()
    } else {
        wnds.get_primary().unwrap()
    };

    // Check if the cursor is inside the window and get its position.
    if let Some(screen_pos) = wnd.cursor_position() {
        // Get the size of the window.
        let window_size = Vec2::new(wnd.width() as f32, wnd.height() as f32);

        // Convert screen position [0..resolution] to ndc [-1..1] (gpu coordinates).
        let ndc = (screen_pos / window_size) * 2.0 - Vec2::ONE;

        // Matrix for undoing the projection and camera transform.
        let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix.inverse();

        // Use it to convert ndc to world-space coordinates.
        let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));

        // Reduce it to a 2D value.
        Some(world_pos.truncate())
    } else {
        None
    }
}

#[derive(Clone, Copy, Default, Inspectable)]
pub enum AimDevice {
    #[default]
    None,
    Gamepad,
    Mouse(Vec2),
}

fn read_player_input(
    keys: Res<Input<KeyCode>>,
    mouse_buttons: Res<Input<MouseButton>>,
    mut cursor_moved: EventReader<CursorMoved>,
    gamepads: Res<Gamepads>,
    pad_buttons: Res<Input<GamepadButton>>,
    axes: Res<Axis<GamepadAxis>>,
    mut egui_ctx: ResMut<EguiContext>,
    mut player_q: Query<(&mut PlayerInput, &GlobalTransform)>,
    wnds: Res<Windows>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
) {
    let (mut input, player_transform) = player_q.single_mut();

    let mut movement = Vec2::ZERO;
    let mut aim = Vec2::ZERO;
    let mut aim_device = input.aim_device;
    let mut shoot = false;
    let mut reset_game = false;

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
                aim_device = AimDevice::Gamepad;
            } else {
                aim_device = AimDevice::None;
            }
        }

        // Shoot
        let shoot_button = GamepadButton(gamepad, GamepadButtonType::RightTrigger2);
        shoot |= pad_buttons.pressed(shoot_button);

        let reset_button = GamepadButton(gamepad, GamepadButtonType::Start);
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
        if let Some(pos) = get_mouse_world_pos(&wnds, camera, camera_transform) {
            aim = (pos - player_transform.translation.truncate()).normalize_or_zero();
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
            velocity.linear = Vec3::ZERO;
        } else {
            velocity.linear = (input.movement * movement.walk_speed).extend(0.0);
        }

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

fn update_player_sprite(
    assets: Res<GameAssets>,
    mut player_q: Query<(&PlayerHealth, &Facing, &PlayerInput, &Knockback, &mut TextureAtlasSprite, &mut Handle<SpriteSheetAnimation>)>,
) {
    for (health, facing, input, knockback, mut sprite, mut anim) in player_q.iter_mut() {
        if facing.dir.x != 0.0 {
            sprite.flip_x = facing.dir.x < 0.0;
        }

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
                visibility.is_visible = true;
            } else {
                visibility.is_visible = false;
            }
        }

        if let Ok((mut transform, mut visibility)) = mouse_crosshair_q.get_single_mut() {
            if let AimDevice::Mouse(pos) = input.aim_device {
                transform.translation = pos.extend(PLAYER_Z + 1.0);
                visibility.is_visible = true;
            } else {
                visibility.is_visible = false;
            }
        }
    }
}

fn apply_post_hit_invuln(
    mut player_q: Query<(&Player, &mut PostHitInvulnerability, &PlayerHealth), Changed<PlayerHealth>>,
    mut hurt_box_q: Query<&mut CollisionLayers>,
) {
    for (player, mut invuln, health) in player_q.iter_mut() {
        if health.current > 0 && health.missing() > 0 {
            invuln.start();

            // Invulnerability started, clear hurt box collision layers.
            if let Ok(mut layers) = hurt_box_q.get_mut(player.hurt_box) {
                *layers = CollisionLayers::none();
            }
        }
    }
}

fn update_post_hit_invuln(
    time: Res<Time>,
    mut player_q: Query<(&Player, &mut PostHitInvulnerability)>,
    mut hurt_box_q: Query<&mut CollisionLayers>,
) {
    let dt = time.delta_seconds();
    for (player, mut invuln) in player_q.iter_mut() {
        if invuln.is_active() {
            invuln.tick(dt);
            if !invuln.is_active() {
                // Invulnerability ended, reset hurt box collision layers.
                if let Ok(mut layers) = hurt_box_q.get_mut(player.hurt_box) {
                    *layers = CollisionLayers::none()
                        .with_groups([CollisionLayer::Player])
                        .with_masks([CollisionLayer::Hit]);
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
    let just_millis = time.time_since_startup().as_millis() % 1000;
    let bucket = just_millis / 100;
    let visible = bucket % 2 == 0;
    for (invuln, mut visibility) in q.iter_mut() {
        if invuln.is_active() {
            visibility.is_visible = visible;
        } else {
            visibility.is_visible = true;
        }
    }
}
