use bevy::prelude::*;
use bevy_egui::{egui, EguiContext, EguiSettings};
use iyes_loopless::prelude::*;

use crate::{
    AppState,
    assets::GameAssets,
    health::PlayerHealth,
    player::Player,
    weapons::{Weapon, WeaponChoice},
};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system(draw_health.run_in_state(AppState::InGame))
            .add_system(draw_weapon.run_in_state(AppState::InGame))
            .add_system(draw_dice.run_in_state(AppState::InGame));
    }
}

fn draw_health(
    mut egui_ctx: ResMut<EguiContext>,
    _egui_settings: Res<EguiSettings>,
    assets: Res<GameAssets>,
    health_q: Query<&PlayerHealth, With<Player>>,
) {
    // TODO: Figure out why health flickers sometimes. Probably an ordering problem.
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

fn draw_dice(
    mut egui_ctx: ResMut<EguiContext>,
    assets: Res<GameAssets>,
) {
    use egui::{Align2, Frame, Window};

    let ctx = egui_ctx.ctx_mut();
    let egui_scale = 2.0;

    let window = Window::new("Dice")
        .anchor(Align2::LEFT_BOTTOM, [20.0, -20.0])
        .auto_sized()
        .title_bar(false)
        .frame(Frame::none());
    window.show(ctx, |ui| {
        ui.horizontal(|ui| {
            let image = &assets.egui_images.dice[0];
            ui.image(image.id, (image.size * egui_scale).to_array());

            let image = &assets.egui_images.dice[1];
            ui.image(image.id, (image.size * egui_scale).to_array());
        });
    });
}

fn draw_weapon(
    mut egui_ctx: ResMut<EguiContext>,
    assets: Res<GameAssets>,
    weapon_q: Query<&Weapon>,
) {
    use egui::{Align2, Color32, Frame, RichText, Window};

    let ctx = egui_ctx.ctx_mut();
    let egui_scale = 2.0;

    if let Ok(weapon) = weapon_q.get_single() {
        let window = Window::new("Weapon")
            .anchor(Align2::LEFT_BOTTOM, [20.0, -60.0])
            .auto_sized()
            .title_bar(false)
            .frame(Frame::none());
        window.show(ctx, |ui| {
            ui.horizontal(|ui| {
                let image = match weapon.equipped {
                    WeaponChoice::Pistol => &assets.egui_images.weapons.pistol,
                    WeaponChoice::RayGun => &assets.egui_images.weapons.ray_gun,
                    WeaponChoice::Shotgun => &assets.egui_images.weapons.shotgun,
                    WeaponChoice::Boomerang => &assets.egui_images.weapons.boomerang,
                    WeaponChoice::Smg => &assets.egui_images.weapons.smg,
                    WeaponChoice::GrenadeLauncher => &assets.egui_images.weapons.grenade_launcher,
                };
                ui.image(image.id, (image.size * egui_scale).to_array());

                ui.add_space(10.0);

                let ammo_text = RichText::new(format!("{} / {}", weapon.ammo, weapon.stats.max_ammo))
                    .color(Color32::WHITE)
                    .size(30.0);
                ui.label(ammo_text);
            });
        });
    }
}
