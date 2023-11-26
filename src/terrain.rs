use std::collections::HashMap;

use bevy::prelude::*;

use crate::{
    AppState,
    assets::GameAssets,
};

const CHUNK_SIZE: f32 = 400.0;

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<SpawnedChunks>()
            .add_systems(Update, spawn_chunks.run_if(in_state(AppState::InGame)));
    }
}

#[derive(Default, Resource)]
pub struct SpawnedChunks(HashMap<IVec2, Entity>);

#[derive(Component)]
struct Chunk;

#[derive(Bundle)]
struct ChunkBundle {
    chunk: Chunk,
    name: Name,
    spatial: SpatialBundle,
}

impl ChunkBundle {
    fn new(chunk_pos: IVec2) -> Self {
        let pos = chunk_pos.as_vec2() * CHUNK_SIZE;
        let transform = Transform::from_translation(pos.extend(0.0));
        Self {
            chunk: Chunk,
            name: Name::new(format!("Chunk({}, {})", chunk_pos.x, chunk_pos.y)),
            spatial: SpatialBundle::from_transform(transform),
        }
    }
}

fn spawn_single_chunk(
    chunk_pos: IVec2,
    commands: &mut Commands,
    assets: &GameAssets,
) -> Entity {
    commands.spawn(ChunkBundle::new(chunk_pos))
        .with_children(|cb| {
            // Spawn grasses.
            let num_grasses = fastrand::u8(15..20);
            for _ in 0..num_grasses {
                let x = fastrand::f32() * CHUNK_SIZE;
                let y = fastrand::f32() * CHUNK_SIZE;
                let index = assets.terrain_indices.grass[fastrand::usize(0..assets.terrain_indices.grass.len())];
                let bundle = SpriteSheetBundle {
                    sprite: TextureAtlasSprite {
                        color: Color::rgba(0.1, 0.4, 0.1, 0.4),
                        index,
                        ..default()
                    },
                    texture_atlas: assets.terrain_atlas.clone(),
                    transform: Transform::from_translation(Vec3::new(x, y, 1.0)),
                    ..default()
                };
                cb.spawn(bundle)
                    .insert(Name::new("Grass"));
            }

            // Spawn dirt.
            let num_dirt = fastrand::u8(8..14);
            for _ in 0..num_dirt {
                let x = fastrand::f32() * CHUNK_SIZE;
                let y = fastrand::f32() * CHUNK_SIZE;
                let index = assets.terrain_indices.dirt[fastrand::usize(0..assets.terrain_indices.dirt.len())];
                let bundle = SpriteSheetBundle {
                    sprite: TextureAtlasSprite {
                        color: Color::rgba(0.4, 0.2, 0.0, 0.4),
                        index,
                        ..default()
                    },
                    texture_atlas: assets.terrain_atlas.clone(),
                    transform: Transform::from_translation(Vec3::new(x, y, 1.0)),
                    ..default()
                };
                cb.spawn(bundle)
                    .insert(Name::new("Dirt"));
            }
        })
        .id()
}

// Check that chunks exist for the current and neighboring chunk positions and spawn any
// missing ones.
pub fn spawn_missing_chunks(
    center_chunk: IVec2,
    commands: &mut Commands,
    assets: &GameAssets,
    spawned_chunks: &mut SpawnedChunks,
) {
    for j in -1..=1 {
        for i in -1..=1 {
            let chunk_pos = center_chunk + IVec2::new(i, j);
            spawned_chunks.0.entry(chunk_pos).or_insert_with(|| {
                debug!("Spawning chunk at {}", chunk_pos);
                spawn_single_chunk(chunk_pos, commands, assets)
            });
        }
    }
}

fn spawn_chunks(
    mut commands: Commands,
    mut last_chunk: Local<IVec2>,
    assets: Res<GameAssets>,
    mut spawned_chunks: ResMut<SpawnedChunks>,
    camera_q: Query<&GlobalTransform, With<Camera>>,
) {
    if let Ok(transform) = camera_q.get_single() {
        // If camera is in a new chunk, spawn any missing chunks around us.
        let camera_pos = transform.translation().truncate();
        let current_chunk = (camera_pos / CHUNK_SIZE).as_ivec2();
        if current_chunk != *last_chunk {
            debug!("Camera entered new chunk (last: {}, current: {}, pos: {}), spawning missing chunks.", *last_chunk, current_chunk, camera_pos);
            spawn_missing_chunks(current_chunk, &mut commands, &assets, &mut spawned_chunks);
            // Update last chunk.
            *last_chunk = current_chunk;
        }
    }
}
