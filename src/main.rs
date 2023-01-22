mod enemy;
mod animation;
mod firework;
mod mario;
mod mushroom;
mod question_block;

use animation::{AnimationTimer, AnimationIndices};
use bevy::{audio::AudioSink, prelude::*};
use enemy::{animate_enemies, move_enemy, enemy_move_event_read_for_mario, EnemyDead, EnemyMoveEvent, enemy_dead_event_read, EnemyType, Enemy, MarioLevelMusicController};
use firework::{animate_fireworks, show_fireworks};
use mario::{move_mario, handle_mario_dead_event, MarioMoveEvent, MarioChangedAsSuperMarioEvent, mario_move_event_read, mario_changed_as_supermario_event_read, get_mario_bundle};
use mushroom::{move_mushroom, magic_mushroom_event_read, MagicMushroomReleaseEvent, MagicMushroom};
use question_block::{animate_question_blocks, hit_questionblock_by_mario, EmptyBlock, QuestionBlock};

fn main() {
    let window = WindowDescriptor {
        title: "Super Mario Rust".to_string(),
        width: 520.0,
        height: 220.0,
        resizable: false,
        ..Default::default()
    };

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    window,
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_startup_system(setup)
        .add_system(animate_enemies)
        .add_system(animate_question_blocks)
        .add_system(animate_fireworks)
        .add_system(show_fireworks)
        .add_system(move_mario)
        .add_system(move_enemy)
        .add_system(handle_mario_dead_event.after(enemy_move_event_read_for_mario))
        .add_system(hit_questionblock_by_mario)
        .add_system(magic_mushroom_event_read.after(hit_questionblock_by_mario))
        .add_system(move_mushroom.after(magic_mushroom_event_read))
        .add_system(mario_move_event_read.after(move_mario))
        .add_system(enemy_move_event_read_for_mario.after(move_enemy))
        .add_system(enemy_dead_event_read.after(enemy_move_event_read_for_mario))
        .add_system(mario_changed_as_supermario_event_read.after(mario_move_event_read))
        .add_event::<MagicMushroomReleaseEvent>()
        .add_event::<MarioChangedAsSuperMarioEvent>()
        .add_event::<MarioMoveEvent>()
        .add_event::<EnemyMoveEvent>()
        .add_event::<EnemyDead>()
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    audio: Res<Audio>,
    audio_sinks: Res<Assets<AudioSink>>,
) {
    let music = asset_server.load("level1_music.ogg");
    let handle = audio_sinks.get_handle(audio.play(music));
    commands.insert_resource(MarioLevelMusicController(handle));

    let background_image = asset_server.load("map.png");

    let questionblock_texture_handle = asset_server.load("question-block.png");
    let questionblock_texture_atlas = TextureAtlas::from_grid(
        questionblock_texture_handle,
        Vec2::new(16.0, 16.0),
        6,
        1,
        None,
        None,
    );
    let questionblock_texture_atlas_handle = texture_atlases.add(questionblock_texture_atlas);

    let emptyblock_texture_handle = asset_server.load("emptyBlock.png");
    let emptyblock_texture_atlas = TextureAtlas::from_grid(
        emptyblock_texture_handle,
        Vec2::new(16.0, 16.0),
        1,
        1,
        None,
        None,
    );
    let emptyblock_texture_atlas_handle = texture_atlases.add(emptyblock_texture_atlas);

    let magicmushroom_texture_handle = asset_server.load("magicMushroom.png");
    let magicmushroom_texture_atlas = TextureAtlas::from_grid(
        magicmushroom_texture_handle,
        Vec2::new(16.0, 16.0),
        1,
        1,
        None,
        None,
    );
    let magicmushroom_texture_atlas_handle = texture_atlases.add(magicmushroom_texture_atlas);

    let goomba_texture_handle = asset_server.load("goomba.png");
    let goomba_texture_atlas = TextureAtlas::from_grid(
        goomba_texture_handle,
        Vec2::new(16.0, 16.0),
        2,
        1,
        None,
        None,
    );
    let goomba_texture_atlas_handle = texture_atlases.add(goomba_texture_atlas);

    let turtle_texture_handle = asset_server.load("turtle.png");
    let turtle_texture_atlas = TextureAtlas::from_grid(
        turtle_texture_handle,
        Vec2::new(16.0, 24.0),
        2,
        1,
        None,
        None,
    );
    let turtle_texture_atlas_handle = texture_atlases.add(turtle_texture_atlas);

    commands.spawn(Camera2dBundle::default());
    commands.spawn(SpriteBundle {
        texture: background_image,
        transform: Transform::from_scale(Vec3::new(1.0, 1.0, 0.0)),
        ..default()
    });
    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: goomba_texture_atlas_handle.clone(),
            sprite: TextureAtlasSprite::new(0),
            transform: Transform::from_xyz(175.0, -78.0, 3.0),
            ..default()
        },
        AnimationIndices { first: 0, last: 1 },
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
        Enemy {
            go_right: false,
            enemy_type: EnemyType::Goomba,
        },
    ));
    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: turtle_texture_atlas_handle.clone(),
            sprite: TextureAtlasSprite::new(0),
            transform: Transform::from_xyz(100.0, -74.0, 3.0),
            ..default()
        },
        AnimationIndices { first: 0, last: 1 },
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
        Enemy {
            go_right: false,
            enemy_type: EnemyType::Turtle,
        },
    ));
    commands.spawn((
        SpriteSheetBundle {
            visibility: Visibility {
                is_visible: (false),
            },
            texture_atlas: magicmushroom_texture_atlas_handle.clone(),
            transform: Transform::from_xyz(2.0, -29.5, 3.0),
            ..default()
        },
        MagicMushroom {
            is_released: false,
            x_reached_max: false,
        },
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
    ));
    commands.spawn((
        SpriteSheetBundle {
            visibility: Visibility {
                is_visible: (false),
            },
            texture_atlas: emptyblock_texture_atlas_handle.clone(),
            transform: Transform::from_xyz(2.0, -29.5, 3.0),
            ..default()
        },
        EmptyBlock,
    ));
    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: questionblock_texture_atlas_handle.clone(),
            sprite: TextureAtlasSprite::new(0),
            transform: Transform::from_xyz(2.0, -29.5, 3.0),
            ..default()
        },
        AnimationIndices { first: 0, last: 5 },
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
        QuestionBlock { is_mushroom: true },
    ));
    commands.spawn((
        SpriteSheetBundle {
            visibility: Visibility {
                is_visible: (false),
            },
            texture_atlas: emptyblock_texture_atlas_handle.clone(),
            transform: Transform::from_xyz(82.5, -29.5, 3.0),
            ..default()
        },
        EmptyBlock,
    ));
    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: questionblock_texture_atlas_handle.clone(),
            sprite: TextureAtlasSprite::new(0),
            transform: Transform::from_xyz(82.5, -29.5, 3.0),
            ..default()
        },
        AnimationIndices { first: 0, last: 5 },
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
        QuestionBlock { is_mushroom: false },
    ));
    commands.spawn((
        SpriteSheetBundle {
            visibility: Visibility {
                is_visible: (false),
            },
            texture_atlas: emptyblock_texture_atlas_handle.clone(),
            transform: Transform::from_xyz(114.5, -29.5, 3.0),
            ..default()
        },
        EmptyBlock,
    ));
    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: questionblock_texture_atlas_handle.clone(),
            sprite: TextureAtlasSprite::new(0),
            transform: Transform::from_xyz(114.5, -29.5, 3.0),
            ..default()
        },
        AnimationIndices { first: 0, last: 5 },
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
        QuestionBlock { is_mushroom: false },
    ));
    commands.spawn((
        SpriteSheetBundle {
            visibility: Visibility {
                is_visible: (false),
            },
            texture_atlas: emptyblock_texture_atlas_handle.clone(),
            transform: Transform::from_xyz(98.2, 34.0, 3.0),
            ..default()
        },
        EmptyBlock,
    ));
    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: questionblock_texture_atlas_handle.clone(),
            sprite: TextureAtlasSprite::new(0),
            transform: Transform::from_xyz(98.2, 34.0, 3.0),
            ..default()
        },
        AnimationIndices { first: 0, last: 5 },
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
        QuestionBlock { is_mushroom: false },
    ));
    commands.spawn(get_mario_bundle(
        asset_server,
        texture_atlases,
        false,
        -250.0,
        -78.0,
    ));
}
