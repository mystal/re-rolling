use bevy::prelude::*;
use bevy_egui::{egui, EguiContext, EguiSettings};
use iyes_loopless::prelude::*;

use crate::{
    AppState,
    assets::GameAssets,
    health::PlayerHealth,
    player::Player,
};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system_to_stage(CoreStage::PostUpdate, draw_hud.run_in_state(AppState::InGame));
    }
}

pub fn draw_hud(
    mut egui_ctx: ResMut<EguiContext>,
    _egui_settings: Res<EguiSettings>,
    assets: Res<GameAssets>,
    health_q: Query<&PlayerHealth, With<Player>>,
) {
    use egui::{Align2, Frame, Window};

    let ctx = egui_ctx.ctx_mut();
    // let egui_scale = egui_settings.scale_factor as f32;
    // TODO: Figure out better way to scale UI.
    let egui_scale = 2.0;

    if let Ok(health) = health_q.get_single() {
        let window = Window::new("PlayerHealth")
            .anchor(Align2::LEFT_TOP, [20.0, 20.0])
            .auto_sized()
            .title_bar(false)
            .frame(Frame::none());
        window.show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Whole hearts.
                let image = &assets.egui_images.whole_heart;
                for _ in 0.. health.current {
                    ui.image(image.id, (image.size * egui_scale).to_array());
                }

                // Empty hearts.
                let image = &assets.egui_images.empty_heart;
                for _ in 0.. health.missing() {
                    ui.image(image.id, (image.size * egui_scale).to_array());
                }
            });
        });
    }
}
