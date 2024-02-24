use std::time::Duration;

use bevy::prelude::*;
use bevy::reflect::TypePath;

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_asset::<Animation>()
            .add_systems(Update, animate_sprites);
    }
}

#[derive(Asset, TypePath, Deref)]
pub struct Animation(benimator::Animation);

impl Animation {
    pub fn from_indices(indices: impl IntoIterator<Item = usize>, frame_time: Duration) -> Self {
        Self(benimator::Animation::from_indices(indices, benimator::FrameRate::from_frame_duration(frame_time)))
    }
}

#[derive(Default, Component, Deref, DerefMut)]
pub struct AnimationState(benimator::State);

#[derive(Default, Component)]
pub struct Play;

pub fn animate_sprites(
    time: Res<Time>,
    animations: Res<Assets<Animation>>,
    mut query: Query<
        (
            &mut AnimationState,
            &mut TextureAtlas,
            &Handle<Animation>,
        ),
        With<Play>,
    >,
) {
    for (mut player, mut atlas, anim_handle) in query.iter_mut() {
        // Get the animation from the handle.
        let Some(animation) = animations.get(anim_handle) else {
            continue;
        };

        // Update the animation state.
        player.update(animation, time.delta());

        // Update the sprite's index into the texture atlas.
        atlas.index = player.frame_index();

        // TODO: Add feature to allow despawning when animation ends.
    }
}
