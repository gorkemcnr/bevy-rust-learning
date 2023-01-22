
use bevy::{prelude::{Res, Component, Query, With, Entity, Transform, EventWriter, Commands, EventReader, Visibility, AssetServer, Audio, Assets, ResMut, Resource, Handle, Vec2, default}, time::{Time, TimerMode, Timer}, sprite::{TextureAtlasSprite, TextureAtlas, SpriteSheetBundle}, audio::AudioSink};
use rand::{distributions::Uniform, prelude::Distribution};

use crate::{AnimationTimer, animation::{AnimationIndices, animate}, mario::{Mario, get_dead_mario_bundle, get_mario_bundle}, firework::Firework};

#[derive(PartialEq)]
pub enum EnemyType {
    Goomba,
    Turtle,
}

pub struct EnemyDead;

#[derive(Resource)]
pub struct MarioLevelMusicController(pub Handle<AudioSink>);

#[derive(Component)]
pub struct Enemy {
    pub go_right: bool,
    pub enemy_type: EnemyType,
}

pub struct EnemyMoveEvent {
    x: f32,
    y: f32,
    entity: Entity,
}

pub fn move_enemy(
    mut enemy_query: Query<
        (Entity, &mut Transform, &mut Enemy, &mut TextureAtlasSprite),
        With<Enemy>,
    >,
    mut enemy_move_event_writer: EventWriter<EnemyMoveEvent>,
) {
    if enemy_query.is_empty() {
        return;
    }

    for (entity, mut transform, mut enemy, mut texture_atlas_sprite) in enemy_query.iter_mut() {
        let position: f32;

        if enemy.go_right {
            if enemy.enemy_type == EnemyType::Turtle {
                texture_atlas_sprite.flip_x = false;
            }
            position = transform.translation.x + 0.5;
        } else {
            if enemy.enemy_type == EnemyType::Turtle {
                texture_atlas_sprite.flip_x = true;
            }
            position = transform.translation.x - 0.5;
        }

        transform.translation.x = position.clamp(-250.0, 180.0);
        if transform.translation.x == -250.0 {
            enemy.go_right = true;
        } else if transform.translation.x == 180.0 {
            enemy.go_right = false;
        }

        enemy_move_event_writer.send(EnemyMoveEvent {
            entity,
            x: transform.translation.x,
            y: transform.translation.y,
        });
    }
}

pub fn animate_enemies(
    time: Res<Time>,
    mut query: Query<
        (
            &mut AnimationTimer,
            &mut TextureAtlasSprite,
            &AnimationIndices,
        ),
        With<Enemy>,
    >,
) {
    for (mut timer, mut sprite, animation_indices) in query.iter_mut() {
        animate(&time, &mut timer, &mut sprite, animation_indices);
    }
}


pub fn enemy_move_event_read_for_mario(
    mut commands: Commands,
    enemy_move_event_reader: EventReader<EnemyMoveEvent>,
    mut enemy_dead_event_writer: EventWriter<EnemyDead>,
    mut mario_query: Query<(&Visibility, &Transform, &mut Mario, Entity), With<Mario>>,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
    audio_sinks: Res<Assets<AudioSink>>,
    music_controller: Res<MarioLevelMusicController>,
    texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    if mario_query.is_empty() {
        return;
    }

    let mario = &mut mario_query.single_mut();

    let enemy_kill_y_limit = if mario.2.is_super_mario { -70.0 } else { -74.0 };

    if let Some(entity) = is_enemy_hit_mario(enemy_move_event_reader, mario.1) {
        if mario.1.translation.y > enemy_kill_y_limit {
            let enemy_kill_sound = asset_server.load("stomp.ogg");
            audio.play(enemy_kill_sound);
            commands.entity(entity).despawn();
            enemy_dead_event_writer.send(EnemyDead);
        } else if !mario.2.is_super_mario {
            if let Some(sink) = audio_sinks.get(&music_controller.0) {
                sink.stop();
            }

            let mariodie = asset_server.load("mariodie.ogg");
            audio.play(mariodie);
            commands.entity(mario.3).despawn();
            commands.spawn(get_dead_mario_bundle(
                asset_server,
                texture_atlases,
                mario.1.translation.x,
                mario.1.translation.y,
            ));
        } else if mario.2.is_super_mario {
            let mariodie = asset_server.load("powerdown.ogg");
            audio.play(mariodie);
            commands.entity(mario.3).despawn();
            commands.spawn(get_mario_bundle(
                asset_server,
                texture_atlases,
                false,
                mario.1.translation.x + 50.0,
                -74.0,
            ));
        }
    }
}

fn is_enemy_hit_mario(
    mut enemy_move_event_reader: EventReader<EnemyMoveEvent>,
    transform: &Transform,
) -> Option<Entity> {
    let mut hit_event: Option<Entity> = None;

    for event in enemy_move_event_reader.iter() {
        if event.x >= transform.translation.x - 5.0
            && event.x <= transform.translation.x + 5.0
            && (event.y >= transform.translation.y - 8.0
                && event.y <= transform.translation.y + 8.0)
        {
            hit_event = Some(event.entity);
        }
    }

    enemy_move_event_reader.clear();

    return hit_event;
}

pub fn enemy_dead_event_read(
    mut commands: Commands,
    enemy_query: Query<Entity, With<Enemy>>,
    enemy_dead_event_read: EventReader<EnemyDead>,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
    audio_sinks: Res<Assets<AudioSink>>,
    music_controller: Res<MarioLevelMusicController>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    if enemy_dead_event_read.is_empty() || !enemy_query.is_empty() {
        return;
    }

    if let Some(sink) = audio_sinks.get(&music_controller.0) {
        sink.stop();
    }

    let mariodie = asset_server.load("stage_clear.ogg");
    audio.play(mariodie);

    let mut rng = rand::thread_rng();
    let x_range = Uniform::from(-250.0..180.0);
    let y_range = Uniform::from(10.0..110.0);

    for _ in 1..8 {
        let x = x_range.sample(&mut rng);
        let y = y_range.sample(&mut rng);

        let texture_handle = asset_server.load("firework.png");
        let texture_atlas =
            TextureAtlas::from_grid(texture_handle, Vec2::new(16.0, 16.0), 16, 1, None, None);

        commands.spawn((
            SpriteSheetBundle {
                visibility: Visibility { is_visible: false },
                texture_atlas: texture_atlases.add(texture_atlas),
                sprite: TextureAtlasSprite::new(0),
                transform: Transform::from_xyz(x, y, 2.0),
                ..default()
            },
            AnimationIndices { first: 0, last: 15 },
            AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
            Firework,
        ));
    }
}
