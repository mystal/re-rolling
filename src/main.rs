#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::prelude::*;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
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
    let cursor = Cursor {
        visible: false,
        ..default()
    };

    // Configure DefaultPlugins.
    let default_plugins = DefaultPlugins
        .set(log::log_plugin())
        .set(ImagePlugin::default_nearest())
        .set(WindowPlugin {
            primary_window: Some(Window {
                title: window::WINDOW_TITLE.into(),
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
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(bevy_egui::EguiPlugin)
        .insert_resource(bevy_egui::EguiSettings {
            // NOTE: Scaling down egui to make in-game UI look chunkier.
            // TODO: Take DPI scaling into account as well.
            scale_factor: (saved_window_state.scale as f32) / 2.0,
            ..default()
        })
        // .insert_resource(RapierConfiguration {
        //     gravity: Vec2::ZERO,
        //     ..default()
        // })
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(1.0))
        .add_plugins(AudioPlugin)

        // App setup
        .init_state::<AppState>()
        .add_plugins((
            window::WindowPlugin::new(saved_window_state),
            animation::AnimationPlugin,
            assets::AssetsPlugin,
            debug::DebugPlugin,
            game::GamePlugin,
        ));

    if ALLOW_EXIT {
        app.add_systems(Update, bevy::window::close_on_esc);
    }

    app.run();
}
