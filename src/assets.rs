use std::time::Duration;

use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_egui::{egui::TextureId, EguiContext};
use iyes_loopless::prelude::*;

use crate::{
    AppState,
    animation::Animation,
};

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_loading_state(
                LoadingState::new(AppState::Loading)
                .continue_to_state(AppState::InGame)
                .with_collection::<GameAssets>()
            )
            .add_exit_system(AppState::Loading, assets_loaded);
    }
}

#[derive(AssetCollection)]
pub struct GameAssets {
    #[asset(texture_atlas(tile_size_x = 16.0, tile_size_y = 16.0, columns = 8, rows = 1))]
    #[asset(path = "player.png")]
    pub player_atlas: Handle<TextureAtlas>,
    pub player_anims: PlayerAnims,

    #[asset(texture_atlas(tile_size_x = 16.0, tile_size_y = 16.0, columns = 13, rows = 1))]
    #[asset(path = "crosshairs.png")]
    pub crosshair_atlas: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 16.0, tile_size_y = 16.0, columns = 5, rows = 1))]
    #[asset(path = "projectiles.png")]
    pub projectile_atlas: Handle<TextureAtlas>,
    pub projectile_indices: ProjectileIndices,

    #[asset(texture_atlas(tile_size_x = 16.0, tile_size_y = 16.0, columns = 4, rows = 1))]
    #[asset(path = "boomerang_projectile.png")]
    pub boomerang_atlas: Handle<TextureAtlas>,
    pub boomerang_anim: Handle<Animation>,

    #[asset(texture_atlas(tile_size_x = 16.0, tile_size_y = 16.0, columns = 8, rows = 2))]
    #[asset(path = "effects.png")]
    pub effects_atlas: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 16.0, tile_size_y = 16.0, columns = 8, rows = 3))]
    #[asset(path = "enemies.png")]
    pub enemy_atlas: Handle<TextureAtlas>,
    pub enemy_indices: EnemyIndices,

    #[asset(texture_atlas(tile_size_x = 16.0, tile_size_y = 16.0, columns = 8, rows = 3))]
    #[asset(path = "terrain.png")]
    pub terrain_atlas: Handle<TextureAtlas>,
    pub terrain_indices: TerrainIndices,

    #[asset(path = "whole_heart.png")]
    pub whole_heart: Handle<Image>,
    #[asset(path = "empty_heart.png")]
    pub empty_heart: Handle<Image>,

    #[asset(path = "dice1.png")]
    pub dice1: Handle<Image>,
    #[asset(path = "dice2.png")]
    pub dice2: Handle<Image>,
    #[asset(path = "dice3.png")]
    pub dice3: Handle<Image>,
    #[asset(path = "dice4.png")]
    pub dice4: Handle<Image>,
    #[asset(path = "dice5.png")]
    pub dice5: Handle<Image>,
    #[asset(path = "dice6.png")]
    pub dice6: Handle<Image>,

    #[asset(path = "pistol.png")]
    pub pistol: Handle<Image>,
    #[asset(path = "ray_gun.png")]
    pub ray_gun: Handle<Image>,
    #[asset(path = "shotgun.png")]
    pub shotgun: Handle<Image>,
    #[asset(path = "boomerang.png")]
    pub boomerang: Handle<Image>,
    #[asset(path = "smg.png")]
    pub smg: Handle<Image>,
    #[asset(path = "grenade_launcher.png")]
    pub grenade_launcher: Handle<Image>,

    pub egui_images: EguiImages,
}

#[derive(Default)]
pub struct PlayerAnims {
    pub idle: Handle<Animation>,
    pub run: Handle<Animation>,
    pub hit_react: Handle<Animation>,
    pub dead: Handle<Animation>,
}

impl PlayerAnims {
    fn new(animations: &mut Assets<Animation>) -> Self {
        let idle = Animation::from_indices(0..=0, Duration::from_millis(200));
        let run = Animation::from_indices(1..=2, Duration::from_millis(150));
        let hit_react = Animation::from_indices(3..=3, Duration::from_millis(100));
        let dead = Animation::from_indices(4..=4, Duration::from_millis(100));
        Self {
            idle: animations.add(idle),
            run: animations.add(run),
            hit_react: animations.add(hit_react),
            dead: animations.add(dead),
        }
    }
}

pub struct ProjectileIndices {
    pub orb: usize,
    pub bullet: usize,
    pub laser: usize,
    pub sparkle: usize,
    pub grenade: usize,
}

impl Default for ProjectileIndices {
    fn default() -> Self {
        Self {
            orb: 0,
            bullet: 1,
            laser: 2,
            sparkle: 3,
            grenade: 4,
        }
    }
}

pub struct EnemyIndices {
    pub rat: usize,
    pub hollow: usize,
    pub snek: usize,
}

impl Default for EnemyIndices {
    fn default() -> Self {
        Self {
            rat: 23,
            hollow: 8,
            snek: 20,
        }
    }
}

pub struct TerrainIndices {
    pub grass: Vec<usize>,
    pub dirt: Vec<usize>,
}

impl Default for TerrainIndices {
    fn default() -> Self {
        Self {
            grass: (5..=7).collect(),
            dirt: (1..=4).collect(),
        }
    }
}

#[derive(Default)]
pub struct EguiWeapons {
    pub pistol: EguiImage,
    pub ray_gun: EguiImage,
    pub shotgun: EguiImage,
    pub boomerang: EguiImage,
    pub smg: EguiImage,
    pub grenade_launcher: EguiImage,
}

#[derive(Default)]
pub struct EguiImage {
    pub id: TextureId,
    pub size: Vec2,
}

#[derive(Default)]
pub struct EguiImages {
    pub whole_heart: EguiImage,
    pub half_heart: EguiImage,
    pub empty_heart: EguiImage,

    pub dice: Vec<EguiImage>,
    pub weapons: EguiWeapons,
}

fn assets_loaded(
    mut egui_ctx: ResMut<EguiContext>,
    mut assets: ResMut<GameAssets>,
    mut animations: ResMut<Assets<Animation>>,
    images: Res<Assets<Image>>,
) {
    debug!("Loaded assets!");

    assets.player_anims = PlayerAnims::new(&mut animations);

    let boomerang_anim = Animation::from_indices(0..=3, Duration::from_millis(150));
    assets.boomerang_anim = animations.add(boomerang_anim);

    if let Some(image) = images.get(&assets.whole_heart) {
        assets.egui_images.whole_heart.id = egui_ctx.add_image(assets.whole_heart.clone_weak());
        assets.egui_images.whole_heart.size = image.size();
    }
    if let Some(image) = images.get(&assets.empty_heart) {
        assets.egui_images.empty_heart.id = egui_ctx.add_image(assets.empty_heart.clone_weak());
        assets.egui_images.empty_heart.size = image.size();
    }

    for handle in [assets.dice1.clone_weak(), assets.dice2.clone_weak(), assets.dice3.clone_weak(), assets.dice4.clone_weak(), assets.dice5.clone_weak(), assets.dice6.clone_weak()] {
        if let Some(image) = images.get(&handle) {
            let egui_image = EguiImage {
                id: egui_ctx.add_image(handle),
                size: image.size(),
            };
            assets.egui_images.dice.push(egui_image);
        }
    }

    if let Some(image) = images.get(&assets.pistol) {
        assets.egui_images.weapons.pistol.id = egui_ctx.add_image(assets.pistol.clone_weak());
        assets.egui_images.weapons.pistol.size = image.size();
    }
    if let Some(image) = images.get(&assets.ray_gun) {
        assets.egui_images.weapons.ray_gun.id = egui_ctx.add_image(assets.ray_gun.clone_weak());
        assets.egui_images.weapons.ray_gun.size = image.size();
    }
    if let Some(image) = images.get(&assets.shotgun) {
        assets.egui_images.weapons.shotgun.id = egui_ctx.add_image(assets.shotgun.clone_weak());
        assets.egui_images.weapons.shotgun.size = image.size();
    }
    if let Some(image) = images.get(&assets.boomerang) {
        assets.egui_images.weapons.boomerang.id = egui_ctx.add_image(assets.boomerang.clone_weak());
        assets.egui_images.weapons.boomerang.size = image.size();
    }
    if let Some(image) = images.get(&assets.smg) {
        assets.egui_images.weapons.smg.id = egui_ctx.add_image(assets.smg.clone_weak());
        assets.egui_images.weapons.smg.size = image.size();
    }
    if let Some(image) = images.get(&assets.grenade_launcher) {
        assets.egui_images.weapons.grenade_launcher.id = egui_ctx.add_image(assets.grenade_launcher.clone_weak());
        assets.egui_images.weapons.grenade_launcher.size = image.size();
    }
}
