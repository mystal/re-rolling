use bevy::prelude::*;
use bevy::time::Stopwatch;
use bevy_kira_audio::prelude::*;
use iyes_loopless::prelude::*;

use crate::{
    AppState,
    assets::{AudioAssets, GameAssets},
    combat,
    enemies,
    health::PlayerHealth,
    player,
    terrain,
    ui,
    weapons,
    window::WindowScale,
};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
         app
            .add_plugin(combat::CombatPlugin)
            .add_plugin(enemies::EnemiesPlugin)
            .add_plugin(player::PlayerPlugin)
            .add_plugin(terrain::TerrainPlugin)
            .add_plugin(ui::UiPlugin)
            .register_type::<Facing>()
            .register_type::<PlayerHealth>()
            .init_resource::<GameTimers>()
            .init_resource::<Bgm>()
            .add_enter_system(AppState::InGame, setup_game)
            .add_system(tick_game_timers.run_in_state(AppState::InGame))
            .add_system(reset_game.run_in_state(AppState::InGame))
            .add_system_to_stage(CoreStage::PostUpdate, camera_follows_player.run_in_state(AppState::InGame))
            .add_system_to_stage(CoreStage::PostUpdate, update_sprite_facing.run_in_state(AppState::InGame))
            .add_system_to_stage(CoreStage::PostUpdate, update_lifetimes.run_in_state(AppState::InGame));
    }
}

#[derive(Resource)]
pub struct GameTimers {
    pub game_time: Stopwatch,
    pub reset_time: Timer,
}

impl Default for GameTimers {
    fn default() -> Self {
        let mut game_time = Stopwatch::new();
        game_time.pause();
        let mut reset_time = Timer::from_seconds(2.0, TimerMode::Once);
        reset_time.pause();

        Self {
            game_time,
            reset_time,
        }
    }
}

#[derive(Default, Resource)]
pub struct Bgm {
    pub handle: Handle<AudioInstance>,
}

fn tick_game_timers(
    time: Res<Time>,
    mut game_timers: ResMut<GameTimers>,
) {
    game_timers.game_time.tick(time.delta());
    game_timers.reset_time.tick(time.delta());
}

fn setup_game(
    mut commands: Commands,
    assets: Res<GameAssets>,
    sounds: Res<AudioAssets>,
    audio: Res<Audio>,
    mut bgm: ResMut<Bgm>,
    window_scale: Res<WindowScale>,
    mut game_timers: ResMut<GameTimers>,
    mut spawned_chunks: ResMut<terrain::SpawnedChunks>,
) {
    game_timers.game_time.reset();
    game_timers.game_time.unpause();

    let mut camera_bundle = Camera2dBundle::default();
    camera_bundle.projection.scale = 1.0 / window_scale.0 as f32;
    commands.spawn(camera_bundle);

    player::spawn_player(Vec2::ZERO, &mut commands, &assets);

    // enemies::spawn_basic_enemy(Vec2::new(300.0, 0.0), &mut commands, &assets);

    commands.spawn((
        enemies::spawner::Spawner::new(50, 1.0),
        Name::new("Spawner"),
    ));

    // Spawn initial terrain chunks.
    terrain::spawn_missing_chunks(IVec2::ZERO, &mut commands, &assets, &mut spawned_chunks);

    bgm.handle = audio.play(sounds.bgm.clone())
        .looped()
        .loop_from(7.381)
        .handle();
}

fn reset_game(
    mut commands: Commands,
    mut game_timers: ResMut<GameTimers>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
    bgm: Res<Bgm>,
    mut player_q: Query<(&player::PlayerInput, &mut Transform, &mut PlayerHealth, &mut weapons::Weapon)>,
    enemy_q: Query<Entity, With<enemies::Enemy>>,
) {
    // If reset_time is finished and player pressed reset input.
    if !game_timers.reset_time.finished() {
        return;
    }

    let (input, mut transform, mut health, mut weapon) = player_q.single_mut();
    if !input.reset_game {
        return;
    }

    // Reset game timers.
    game_timers.game_time.reset();
    game_timers.game_time.unpause();
    game_timers.reset_time.reset();
    game_timers.reset_time.pause();

    // Reset player.
    transform.translation.x = 0.0;
    transform.translation.y = 0.0;
    health.current = health.max;
    *weapon = weapons::Weapon::new(weapons::WeaponChoice::random());

    // Kill all enemies.
    for entity in enemy_q.iter() {
        commands.entity(entity)
            .insert(enemies::Death);
    }

    if let Some(instance) = audio_instances.get_mut(&bgm.handle) {
        instance.seek_to(0.0);
    }
}

#[derive(Component)]
pub struct Crosshair;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Facing {
    pub dir: Vec2,
}

impl Default for Facing {
    fn default() -> Self {
        Self {
            dir: Vec2::X,
        }
    }
}

#[derive(Component)]
pub struct Lifetime {
    pub lifetime: f32,
    pub remaining: f32,
}

impl Lifetime {
    pub fn new(seconds: f32) -> Self {
        Self {
            lifetime: seconds,
            remaining: seconds,
        }
    }
}

fn update_lifetimes(
    mut commands: Commands,
    time: Res<Time>,
    mut q: Query<(Entity, &mut Lifetime)>,
) {
    let dt = time.delta_seconds();
    for (entity, mut lifetime) in q.iter_mut() {
        lifetime.remaining = (lifetime.remaining - dt).max(0.0);
        if lifetime.remaining <= 0.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn update_sprite_facing(
    mut q: Query<(&mut TextureAtlasSprite, &Facing)>,
) {
    for (mut sprite, facing) in q.iter_mut() {
        if facing.dir.x != 0.0 {
            sprite.flip_x = facing.dir.x < 0.0;
        }
    }
}

fn camera_follows_player(
    player_q: Query<&Transform, (With<player::Player>, Changed<Transform>)>,
    mut camera_q: Query<&mut Transform, (With<Camera>, Without<player::Player>)>,
) {
    if let (Ok(mut camera_transform), Ok(player_transform)) = (camera_q.get_single_mut(), player_q.get_single()) {
        camera_transform.translation.x = player_transform.translation.x;
        camera_transform.translation.y = player_transform.translation.y;
    }
}
