#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::prelude::*;
use bevy::render::texture::ImageSettings;
use bevy::window::WindowMode;
use iyes_loopless::prelude::*;

mod animation;
mod assets;
mod combat;
mod debug;
mod enemies;
mod game;
mod health;
mod log;
mod physics;
mod player;
mod terrain;
mod ui;
mod weapons;
mod window;

// TODO: Choose a good size for this game.
// const GAME_SIZE: (f32, f32) = (320.0, 180.0);
const DEFAULT_SCALE: u8 = 2;
const GAME_LOGIC_FPS: u8 = 60;
const GAME_LOGIC_FRAME_TIME: f32 = 1.0 / GAME_LOGIC_FPS as f32;
const ALLOW_EXIT: bool = cfg!(not(target_arch = "wasm32"));

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum AppState {
    Loading,
    InGame,
}

fn main() {
    // When building for WASM, print panics to the browser console.
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    // TODO: Try to initialize logging before this. Maybe we can also make this code run in a plugin.
    let saved_window_state = window::load_window_state();

    let window_position = saved_window_state.position
        .map(|pos| WindowPosition::At(pos.as_vec2()))
        .unwrap_or(WindowPosition::Automatic);

    let mut app = App::new();

    app
        // Added first so logging is configured properly when DefaultPlugins are added.
        .add_plugin(log::LogPlugin)

        .insert_resource(WindowDescriptor {
            title: "Re-Rolling!".into(),
            // width: GAME_SIZE.0 * saved_window_state.scale as f32,
            // height: GAME_SIZE.1 * saved_window_state.scale as f32,
            resizable: false,
            position: window_position,
            mode: WindowMode::Windowed,
            cursor_visible: false,
            ..default()
        })
        .insert_resource(ImageSettings::default_nearest())
        .insert_resource(ClearColor(Color::rgb_u8(160, 160, 160)))

        // External plugins
        .add_plugins(DefaultPlugins)
        .add_plugin(bevy_egui::EguiPlugin)
        .insert_resource(bevy_egui::EguiSettings {
            // TODO: Take DPI scaling into account as well.
            scale_factor: (saved_window_state.scale as f64) / (DEFAULT_SCALE as f64),
            ..default()
        })
        .add_plugin(heron::PhysicsPlugin::default())
        // .add_plugin(bevy_tweening::TweeningPlugin)

        // App setup
        .insert_resource(window::WindowScale(saved_window_state.scale))
        .add_loopless_state(AppState::Loading)
        .add_plugin(animation::AnimationPlugin)
        .add_plugin(assets::AssetsPlugin)
        .add_plugin(debug::DebugPlugin)
        .add_plugin(game::GamePlugin)
        .add_plugin(window::WindowPlugin);

    if ALLOW_EXIT {
        app.add_system(bevy::window::close_on_esc);
    }

    app.run();
}
