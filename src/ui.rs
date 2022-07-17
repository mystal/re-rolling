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
    time: Res<Time>,
    weapon_q: Query<&Weapon>,
) {
    use egui::{Align2, Frame, Window};

    let ctx = egui_ctx.ctx_mut();
    let egui_scale = 2.0;

    if let Ok(weapon) = weapon_q.get_single() {
        let window = Window::new("Dice")
            .anchor(Align2::LEFT_BOTTOM, [20.0, -20.0])
            .auto_sized()
            .title_bar(false)
            .frame(Frame::none());
        window.show(ctx, |ui| {
            ui.horizontal(|ui| {
                if !weapon.reloading {
                    let image = &assets.egui_images.dice[weapon.equipped as usize];
                    ui.image(image.id, (image.size * egui_scale).to_array());
                } else {
                    let just_millis = time.time_since_startup().as_millis() % 1000;
                    let bucket = (just_millis as f32 / 1000.0) * 6.0;
                    let image = &assets.egui_images.dice[bucket as usize];
                    ui.image(image.id, (image.size * egui_scale).to_array());
                }
            });
        });
    }
}

fn draw_weapon(
    mut egui_ctx: ResMut<EguiContext>,
    assets: Res<GameAssets>,
    time: Res<Time>,
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
                if !weapon.reloading {
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
                } else {
                    let just_millis = time.time_since_startup().as_millis() % 1000;
                    let bucket = (just_millis as f32 / 1000.0) * 6.0;
                    let image = match bucket as u32 {
                        2 => &assets.egui_images.weapons.pistol,
                        5 => &assets.egui_images.weapons.ray_gun,
                        3 => &assets.egui_images.weapons.shotgun,
                        4 => &assets.egui_images.weapons.boomerang,
                        1 => &assets.egui_images.weapons.smg,
                        0 => &assets.egui_images.weapons.grenade_launcher,
                        _ => &assets.egui_images.weapons.pistol,
                    };

                    ui.image(image.id, (image.size * egui_scale).to_array());

                    ui.add_space(10.0);

                    let ammo_text = RichText::new("Re-Rolling...")
                        .color(Color32::WHITE)
                        .size(30.0);
                    ui.label(ammo_text);
                }
            });
        });
    }
}
