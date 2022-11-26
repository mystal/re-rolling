use bevy::prelude::*;
use bevy_egui::EguiContext;
use bevy_inspector_egui::{WorldInspectorParams, WorldInspectorPlugin};
use bevy_rapier2d::render::DebugRenderContext;
use iyes_loopless::prelude::*;

use crate::{
    AppState,
    player::PlayerInput,
    weapons::{Weapon, WeaponChoice},
};

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(WorldInspectorParams {
                enabled: false,
                ..default()
            })
            .add_plugin(WorldInspectorPlugin::new())

            .insert_resource(DebugState::default())
            // TODO: Remove once bug for configuring this when adding the RapierDebugRenderPlugin works.
            .add_startup_system(disable_rapier_debug_render)
            .add_system(toggle_world_inspector)
            .add_system(toggle_physics_debug_render)
            .add_system(select_weapon.run_in_state(AppState::InGame).before("player_input"))
            .add_system_to_stage(CoreStage::Last, update_mouse_cursor);
    }
}

#[derive(Default, Resource)]
struct DebugState {
    enabled: bool,
}

fn update_mouse_cursor(
    mut windows: ResMut<Windows>,
    debug_state: Res<DebugState>,
    mut egui_ctx: ResMut<EguiContext>,
) {
    if let Some(window) = windows.get_primary_mut() {
        // TODO: Make UI egui windows non-interactable and remove the debug_state.enabled check.
        let show_cursor = debug_state.enabled && egui_ctx.ctx_mut().wants_pointer_input();
        window.set_cursor_visibility(show_cursor);
    }
}

fn toggle_world_inspector(
    keys: ResMut<Input<KeyCode>>,
    mut debug_state: ResMut<DebugState>,
    mut inspector_params: ResMut<WorldInspectorParams>,
    mut egui_ctx: ResMut<EguiContext>,
) {
    if egui_ctx.ctx_mut().wants_keyboard_input() {
        return;
    }

    if keys.just_pressed(KeyCode::Back) {
        debug_state.enabled = !debug_state.enabled;
        inspector_params.enabled = !inspector_params.enabled;
    }
}

fn select_weapon(
    keys: ResMut<Input<KeyCode>>,
    mut egui_ctx: ResMut<EguiContext>,
    mut player_q: Query<&mut Weapon, With<PlayerInput>>,
) {
    if egui_ctx.ctx_mut().wants_keyboard_input() {
        return;
    }

    let mut weapon = player_q.single_mut();

    for key in keys.get_just_pressed() {
        let choice = match key {
            KeyCode::Key1 => WeaponChoice::Pistol,
            KeyCode::Key2 => WeaponChoice::RayGun,
            KeyCode::Key3 => WeaponChoice::Shotgun,
            KeyCode::Key4 => WeaponChoice::Boomerang,
            KeyCode::Key5 => WeaponChoice::Smg,
            KeyCode::Key6 => WeaponChoice::GrenadeLauncher,
            _ => continue,
        };
        *weapon = Weapon::new(choice);
    }
}

fn toggle_physics_debug_render(
    keys: ResMut<Input<KeyCode>>,
    mut egui_ctx: ResMut<EguiContext>,
    mut debug_render_context: ResMut<DebugRenderContext>,
) {
    if egui_ctx.ctx_mut().wants_keyboard_input() {
        return;
    }

    if keys.just_pressed(KeyCode::Key0) {
        debug_render_context.enabled = !debug_render_context.enabled;
    }
}

fn disable_rapier_debug_render(
    mut debug_render_context: ResMut<DebugRenderContext>,
) {
    debug_render_context.enabled = false;
}
