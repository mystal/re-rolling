use bevy::prelude::*;
use bevy_ui_dsl::*;

use crate::{
    AppState,
    ui::classes::*,
    window::WindowState,
};

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(AppState::InGame), spawn_ui);
    }
}

#[derive(Component)]
struct HudRoot;

#[derive(Component)]
struct TimerText;

#[derive(Component)]
struct HealthUi;

#[derive(Component)]
struct WeaponIcon;

#[derive(Component)]
struct AmmoText;

#[derive(Component)]
struct DieIcon;

fn spawn_ui(
    mut commands: Commands,
    window_state: Res<WindowState>,
    mut ui_scale: ResMut<UiScale>,
    asset_server: Res<AssetServer>,
) {
    ui_scale.0 = window_state.scale as f32;

    rooti(c_root, &asset_server, &mut commands, (HudRoot, Name::new("HudRoot")), |p| {
        nodei(c_timer, Name::new("Timer"), p, |p| {
            texti(format!("{:03.0}", 0), c_timer_text, c_timer_font, TimerText, p);
        });
        nodei(c_health_ui, (HealthUi, Name::new("HealthUi")), p, |p| {
            image(c_whole_heart, p);
            image(c_whole_heart, p);
            image(c_whole_heart, p);
            image(c_whole_heart, p);
        });
        // nodei(c_cat_tracker, Name::new("CatTracker"), p, |p| {
        //     image(c_cat_face, p);
        //     // TODO: Add a drop shadow to the text.
        //     texti("00/00", c_tracker_text, c_font_tracker, CatTracker, p);
        // });
    });
}
