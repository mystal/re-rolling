use std::time::Duration;

use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy_asset_loader::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_egui::{egui::TextureId, EguiContexts};
use bevy_kira_audio::AudioSource;
use serde::Deserialize;

use crate::{
    AppState,
    animation::Animation,
};

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(RonAssetPlugin::<AudioConfig>::new(&["audio.ron"]))
            .add_loading_state(
                LoadingState::new(AppState::Loading)
                    .continue_to_state(AppState::InGame)
                    .with_dynamic_assets_file::<StandardDynamicAssetCollection>("audio/audio.assets.ron")
                    .load_collection::<GameAssets>()
                    .load_collection::<AudioAssets>()
            )
            .add_systems(OnExit(AppState::Loading), assets_loaded);
    }
}

#[derive(Resource, AssetCollection)]
pub struct GameAssets {
    #[asset(path = "player.png")]
    pub player: Handle<Image>,
    #[asset(texture_atlas_layout(tile_size_x = 16.0, tile_size_y = 16.0, columns = 8, rows = 1))]
    pub player_atlas: Handle<TextureAtlasLayout>,
    pub player_anims: PlayerAnims,

    #[asset(path = "crosshairs.png")]
    pub crosshairs: Handle<Image>,
    #[asset(texture_atlas_layout(tile_size_x = 16.0, tile_size_y = 16.0, columns = 13, rows = 1))]
    pub crosshairs_atlas: Handle<TextureAtlasLayout>,

    #[asset(path = "projectiles.png")]
    pub projectiles: Handle<Image>,
    #[asset(texture_atlas_layout(tile_size_x = 16.0, tile_size_y = 16.0, columns = 5, rows = 1))]
    pub projectile_atlas: Handle<TextureAtlasLayout>,
    pub projectile_indices: ProjectileIndices,

    #[asset(path = "boomerang_projectile.png")]
    pub boomerang_projectile: Handle<Image>,
    #[asset(texture_atlas_layout(tile_size_x = 16.0, tile_size_y = 16.0, columns = 4, rows = 1))]
    pub boomerang_atlas: Handle<TextureAtlasLayout>,
    pub boomerang_anim: Handle<Animation>,

    #[asset(path = "effects.png")]
    pub effects: Handle<Image>,
    #[asset(texture_atlas_layout(tile_size_x = 16.0, tile_size_y = 16.0, columns = 8, rows = 2))]
    pub effects_atlas: Handle<TextureAtlasLayout>,

    #[asset(path = "explosions.png")]
    pub explosions: Handle<Image>,
    #[asset(texture_atlas_layout(tile_size_x = 16.0, tile_size_y = 16.0, columns = 4, rows = 2))]
    pub explosions_atlas: Handle<TextureAtlasLayout>,
    pub explosion_anim: Handle<Animation>,

    #[asset(path = "enemies.png")]
    pub enemy: Handle<Image>,
    #[asset(texture_atlas_layout(tile_size_x = 16.0, tile_size_y = 16.0, columns = 8, rows = 3))]
    pub enemy_atlas: Handle<TextureAtlasLayout>,
    pub enemy_indices: EnemyIndices,

    #[asset(path = "terrain.png")]
    pub terrain: Handle<Image>,
    #[asset(texture_atlas_layout(tile_size_x = 16.0, tile_size_y = 16.0, columns = 8, rows = 3))]
    pub terrain_atlas: Handle<TextureAtlasLayout>,
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

#[derive(Deserialize)]
pub struct SfxVolumes {
    pub pistol: f32,
    pub raygun: f32,
    pub shotgun: f32,
    pub smg: f32,
    pub boomerang: f32,
    pub grenade: f32,
    pub grenade_explosion: f32,
}

impl Default for SfxVolumes {
    fn default() -> Self {
        Self {
            pistol: 1.0,
            raygun: 1.0,
            shotgun: 1.0,
            smg: 1.0,
            boomerang: 1.0,
            grenade: 1.0,
            grenade_explosion: 1.0,
        }
    }
}

#[derive(Default, Deserialize, Asset, TypePath)]
pub struct AudioConfig {
    pub bgm_loop_time: f64,
    #[serde(default)]
    pub sfx_volumes: SfxVolumes,
}

#[derive(Resource, AssetCollection)]
pub struct AudioAssets {
    #[asset(key = "pistol")]
    pub pistol: Handle<AudioSource>,
    #[asset(key = "raygun")]
    pub raygun: Handle<AudioSource>,
    #[asset(key = "shotgun")]
    pub shotgun: Handle<AudioSource>,
    #[asset(key = "smg")]
    pub smg: Handle<AudioSource>,
    #[asset(key = "boomerang")]
    pub boomerang: Handle<AudioSource>,
    #[asset(key = "grenade")]
    pub grenade: Handle<AudioSource>,
    #[asset(key = "grenade_explosion")]
    pub grenade_explosion: Handle<AudioSource>,

    #[asset(key = "bgm")]
    pub bgm: Handle<AudioSource>,

    #[asset(path = "audio/config.audio.ron")]
    pub config: Handle<AudioConfig>,
}

fn assets_loaded(
    mut egui_ctx: EguiContexts,
    mut assets: ResMut<GameAssets>,
    mut animations: ResMut<Assets<Animation>>,
    images: Res<Assets<Image>>,
) {
    debug!("Loaded assets!");

    assets.player_anims = PlayerAnims::new(&mut animations);

    let boomerang_anim = Animation::from_indices(0..=3, Duration::from_millis(150));
    assets.boomerang_anim = animations.add(boomerang_anim);

    let explosion_anim = Animation::from_indices(0..=3, Duration::from_millis(100));
    assets.explosion_anim = animations.add(explosion_anim);

    if let Some(image) = images.get(&assets.whole_heart) {
        assets.egui_images.whole_heart.id = egui_ctx.add_image(assets.whole_heart.clone_weak());
        assets.egui_images.whole_heart.size = image.size().as_vec2();
    }
    if let Some(image) = images.get(&assets.empty_heart) {
        assets.egui_images.empty_heart.id = egui_ctx.add_image(assets.empty_heart.clone_weak());
        assets.egui_images.empty_heart.size = image.size().as_vec2();
    }

    for handle in [assets.dice1.clone_weak(), assets.dice2.clone_weak(), assets.dice3.clone_weak(), assets.dice4.clone_weak(), assets.dice5.clone_weak(), assets.dice6.clone_weak()] {
        if let Some(image) = images.get(&handle) {
            let egui_image = EguiImage {
                id: egui_ctx.add_image(handle),
                size: image.size().as_vec2(),
            };
            assets.egui_images.dice.push(egui_image);
        }
    }

    if let Some(image) = images.get(&assets.pistol) {
        assets.egui_images.weapons.pistol.id = egui_ctx.add_image(assets.pistol.clone_weak());
        assets.egui_images.weapons.pistol.size = image.size().as_vec2();
    }
    if let Some(image) = images.get(&assets.ray_gun) {
        assets.egui_images.weapons.ray_gun.id = egui_ctx.add_image(assets.ray_gun.clone_weak());
        assets.egui_images.weapons.ray_gun.size = image.size().as_vec2();
    }
    if let Some(image) = images.get(&assets.shotgun) {
        assets.egui_images.weapons.shotgun.id = egui_ctx.add_image(assets.shotgun.clone_weak());
        assets.egui_images.weapons.shotgun.size = image.size().as_vec2();
    }
    if let Some(image) = images.get(&assets.boomerang) {
        assets.egui_images.weapons.boomerang.id = egui_ctx.add_image(assets.boomerang.clone_weak());
        assets.egui_images.weapons.boomerang.size = image.size().as_vec2();
    }
    if let Some(image) = images.get(&assets.smg) {
        assets.egui_images.weapons.smg.id = egui_ctx.add_image(assets.smg.clone_weak());
        assets.egui_images.weapons.smg.size = image.size().as_vec2();
    }
    if let Some(image) = images.get(&assets.grenade_launcher) {
        assets.egui_images.weapons.grenade_launcher.id = egui_ctx.add_image(assets.grenade_launcher.clone_weak());
        assets.egui_images.weapons.grenade_launcher.size = image.size().as_vec2();
    }
}
