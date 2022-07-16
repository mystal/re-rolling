use std::time::Duration;

use benimator::SpriteSheetAnimation;
use bevy::prelude::*;
use bevy_asset_loader::{AssetCollection, AssetLoader};
use iyes_loopless::prelude::*;

use crate::AppState;

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        AssetLoader::new(AppState::Loading)
            .continue_to_state(AppState::InGame)
            .with_collection::<GameAssets>()
            .build(app);
        app.add_exit_system(AppState::Loading, assets_loaded);
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

    #[asset(texture_atlas(tile_size_x = 16.0, tile_size_y = 16.0, columns = 4, rows = 1))]
    #[asset(path = "projectiles.png")]
    pub projectile_atlas: Handle<TextureAtlas>,
    pub projectile_indices: ProjectileIndices,

    #[asset(texture_atlas(tile_size_x = 16.0, tile_size_y = 16.0, columns = 8, rows = 3))]
    #[asset(path = "enemies.png")]
    pub enemy_atlas: Handle<TextureAtlas>,
    pub enemy_indices: EnemyIndices,
}

#[derive(Default)]
pub struct PlayerAnims {
    pub idle: Handle<SpriteSheetAnimation>,
    pub run: Handle<SpriteSheetAnimation>,
    pub hit_react: Handle<SpriteSheetAnimation>,
    pub dead: Handle<SpriteSheetAnimation>,
}

impl PlayerAnims {
    fn new(animations: &mut Assets<SpriteSheetAnimation>) -> Self {
        let idle = SpriteSheetAnimation::from_range(0..=0, Duration::from_millis(200));
        let run = SpriteSheetAnimation::from_range(1..=2, Duration::from_millis(150));
        let hit_react = SpriteSheetAnimation::from_range(3..=3, Duration::from_millis(100));
        let dead = SpriteSheetAnimation::from_range(4..=4, Duration::from_millis(100));
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
}

impl Default for ProjectileIndices {
    fn default() -> Self {
        Self {
            orb: 0,
            bullet: 1,
            laser: 2,
            sparkle: 3,
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

fn assets_loaded(
    mut assets: ResMut<GameAssets>,
    mut animations: ResMut<Assets<SpriteSheetAnimation>>,
) {
    debug!("Loaded assets!");

    assets.player_anims = PlayerAnims::new(&mut animations);
}
