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

fn assets_loaded(
    mut assets: ResMut<GameAssets>,
    // mut egui_ctx: ResMut<EguiContext>,
    mut animations: ResMut<Assets<SpriteSheetAnimation>>,
    images: Res<Assets<Image>>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
) {
    debug!("Loaded assets!");

    assets.player_anims = PlayerAnims::new(&mut animations);

    // if let Some(image) = images.get(&assets.whole_heart) {
    //     assets.egui.whole_heart.id = egui_ctx.add_image(assets.whole_heart.clone_weak());
    //     assets.egui.whole_heart.size = image.size();
    // }
    // if let Some(image) = images.get(&assets.empty_heart) {
    //     assets.egui.empty_heart.id = egui_ctx.add_image(assets.empty_heart.clone_weak());
    //     assets.egui.empty_heart.size = image.size();
    // }
}
