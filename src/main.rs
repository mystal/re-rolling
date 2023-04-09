#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::prelude::*;
use bevy::window::{Cursor, WindowMode};
use bevy_kira_audio::AudioPlugin;
use bevy_rapier2d::prelude::*;

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
const DEFAULT_SCALE: u8 = 3;
const GAME_LOGIC_FPS: u8 = 60;
const GAME_LOGIC_FRAME_TIME: f32 = 1.0 / GAME_LOGIC_FPS as f32;
const ALLOW_EXIT: bool = cfg!(not(target_arch = "wasm32"));

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
enum AppState {
    #[default]
    Loading,
    InGame,
}

fn main() {
    // When building for WASM, print panics to the browser console.
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    // TODO: Try to initialize logging before this. Maybe we can also make this code run in a plugin.
    let saved_window_state = window::load_window_state();
    let cursor = {
        let mut c = Cursor::default();
        c.visible = false;
        c
    };

    // Configure DefaultPlugins.
    let default_plugins = DefaultPlugins
        .set(log::log_plugin())
        .set(ImagePlugin::default_nearest())
        .set(WindowPlugin {
            primary_window: Some(Window {
                title: "Re-Rolling!".into(),
                // width: GAME_SIZE.0 * saved_window_state.scale as f32,
                // height: GAME_SIZE.1 * saved_window_state.scale as f32,
                resizable: false,
                position: saved_window_state.position,
                mode: WindowMode::Windowed,
                cursor,
                ..default()
            }),
            ..default()
        });

    let mut app = App::new();
    app
        .insert_resource(ClearColor(Color::rgb_u8(160, 160, 160)))

        // External plugins
        .add_plugins(default_plugins)
        .add_plugin(bevy_egui::EguiPlugin)
        .insert_resource(bevy_egui::EguiSettings {
            // NOTE: Scaling down egui to make in-game UI look chunkier.
            // TODO: Take DPI scaling into account as well.
            scale_factor: (saved_window_state.scale as f64) / (2.0 as f64),
            ..default()
        })
        .insert_resource(RapierConfiguration {
            gravity: Vec2::ZERO,
            ..default()
        })
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(1.0))
        .add_plugin(AudioPlugin)

        // App setup
        .add_state::<AppState>()
        .add_plugin(window::WindowPlugin::new(saved_window_state))
        .add_plugin(animation::AnimationPlugin)
        .add_plugin(assets::AssetsPlugin)
        .add_plugin(debug::DebugPlugin)
        .add_plugin(game::GamePlugin);

    if ALLOW_EXIT {
        app.add_system(bevy::window::close_on_esc);
    }

    app.run();
}
