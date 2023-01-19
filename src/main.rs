use bevy::{audio::AudioSink, prelude::*};
use rand::distributions::{Distribution, Uniform};
use std::time::Duration;

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

#[derive(PartialEq)]
enum EnemyType {
    Goomba,
    Turtle,
}

#[derive(Resource)]
struct MarioLevelMusicController(Handle<AudioSink>);

struct MagicMushroomReleaseEvent {
    x: f32,
    y: f32,
}

struct MarioMoveEvent {
    x: f32,
    y: f32,
}

struct EnemyMoveEvent {
    x: f32,
    y: f32,
    entity: Entity,
}

struct MarioChangedAsSuperMarioEvent {
    x: f32,
    y: f32,
}

struct EnemyDead;

#[derive(Component)]
struct MarioDead {
    go_up: bool,
}

#[derive(Component)]
struct Mario {
    dont_go_up_until_settle: bool,
    is_super_mario: bool,
}

#[derive(Component)]
struct Firework;

#[derive(Component)]
struct EmptyBlock;

#[derive(Component)]
struct Enemy {
    go_right: bool,
    enemy_type: EnemyType,
}

#[derive(Component)]
struct MagicMushroom {
    is_released: bool,
    x_reached_max: bool,
}

#[derive(Component)]
struct QuestionBlock {
    is_mushroom: bool,
}

#[derive(Component)]
struct AnimationIndices {
    first: usize,
    last: usize,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

fn show_fireworks(
    mut query: Query<(&mut AnimationTimer, &mut Visibility), With<Firework>>,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
) {
    if query.is_empty() {
        return;
    }

    if let Some(mut firework) = query.iter_mut().next() {
        if !firework.1.is_visible {
            firework.1.is_visible = true;
            let firework_sound = asset_server.load("firework.ogg");
            audio.play(firework_sound);
        }
    }
}

fn animate_fireworks(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<
        (
            &mut AnimationTimer,
            &mut TextureAtlasSprite,
            &AnimationIndices,
            &Visibility,
            Entity,
        ),
        With<Firework>,
    >,
) {
    for (mut timer, mut sprite, animation_indices, visibility, entity) in query.iter_mut() {
        if visibility.is_visible {
            animate(&time, &mut timer, &mut sprite, animation_indices);

            if sprite.index == 15 {
                commands.entity(entity).despawn();
            }
        }
    }
}

fn animate_enemies(
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

fn animate_question_blocks(
    time: Res<Time>,
    mut query: Query<
        (
            &mut AnimationTimer,
            &mut TextureAtlasSprite,
            &AnimationIndices,
        ),
        With<QuestionBlock>,
    >,
) {
    for (mut timer, mut sprite, animation_indices) in query.iter_mut() {
        animate(&time, &mut timer, &mut sprite, animation_indices);
    }
}

fn animate(
    time: &Res<Time>,
    timer: &mut AnimationTimer,
    sprite: &mut TextureAtlasSprite,
    animation_indices: &AnimationIndices,
) {
    timer.tick(time.delta());
    if timer.just_finished() {
        sprite.index = (sprite.index + 1) % (animation_indices.last + 1);
    }
}

fn move_mushroom(
    mut query: Query<
        (
            &mut AnimationTimer,
            &mut Transform,
            &mut Visibility,
            &mut MagicMushroom,
        ),
        With<MagicMushroom>,
    >,
) {
    for (mut timer, mut transform, mut visibility, mut magic_mushroom) in &mut query.iter_mut() {
        if visibility.is_visible {
            timer.tick(Duration::from_secs_f32(1.0));
            if timer.just_finished() {
                if !magic_mushroom.is_released {
                    transform.translation.y += 0.4;
                    if transform.translation.y >= -11.0 {
                        magic_mushroom.is_released = true;
                    }
                } else {
                    let mut direction_x = 0.5;
                    let mut direction_y = 0.1;

                    if transform.translation.x >= 12.0 {
                        direction_y = 0.5;
                    }

                    if magic_mushroom.x_reached_max {
                        direction_x = -0.5;
                    }

                    let mushroom_position_x = transform.translation.x + direction_x;
                    let mushroom_position_y = transform.translation.y - direction_y;

                    transform.translation.x = mushroom_position_x.clamp(-280.0, 180.0);
                    transform.translation.y = mushroom_position_y.clamp(-78.0, -12.0);

                    if transform.translation.x == 180.0 {
                        magic_mushroom.x_reached_max = true;
                    }

                    if transform.translation.x == -280.0 {
                        visibility.is_visible = false;
                    }
                }
            }
        }
    }
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

fn get_mario_bundle(
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    is_super_mario: bool,
    x: f32,
    y: f32,
) -> (
    bevy::prelude::SpriteSheetBundle,
    AnimationIndices,
    AnimationTimer,
    Mario,
) {
    let mario_texture_handle = if !is_super_mario {
        asset_server.load("mario-walk.png")
    } else {
        asset_server.load("super-mario-walk.png")
    };
    let mario_texture_atlas = TextureAtlas::from_grid(
        mario_texture_handle,
        if !is_super_mario {
            Vec2::new(15.0, 16.0)
        } else {
            Vec2::new(16.0, 32.0)
        },
        3,
        1,
        None,
        None,
    );

    return (
        SpriteSheetBundle {
            texture_atlas: texture_atlases.add(mario_texture_atlas),
            sprite: TextureAtlasSprite::new(0),
            transform: Transform::from_xyz(x, y, 2.0),
            ..default()
        },
        AnimationIndices { first: 0, last: 2 },
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
        Mario {
            is_super_mario,
            dont_go_up_until_settle: false,
        },
    );
}

fn get_dead_mario_bundle(
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    x: f32,
    y: f32,
) -> (bevy::prelude::SpriteSheetBundle, AnimationTimer, MarioDead) {
    let mario_texture_handle = asset_server.load("mario_dead.png");

    let mario_texture_atlas = TextureAtlas::from_grid(
        mario_texture_handle,
        Vec2::new(15.0, 16.0),
        1,
        1,
        None,
        None,
    );

    return (
        SpriteSheetBundle {
            texture_atlas: texture_atlases.add(mario_texture_atlas),
            sprite: TextureAtlasSprite::new(0),
            transform: Transform::from_xyz(x, y, 2.0),
            ..default()
        },
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
        MarioDead { go_up: true },
    );
}

fn magic_mushroom_event_read(
    mut magic_mushroom_event_reader: EventReader<MagicMushroomReleaseEvent>,
    mut magic_mushroom_query: Query<(&mut Visibility, &Transform), With<MagicMushroom>>,
) {
    for event in magic_mushroom_event_reader.iter() {
        for (mut magic_mushroom_visibility, magic_mushroom_transform) in
            magic_mushroom_query.iter_mut()
        {
            if event.x == magic_mushroom_transform.translation.x
                && event.y == magic_mushroom_transform.translation.y
            {
                magic_mushroom_visibility.is_visible = true;
            }
        }
    }
}

fn mario_changed_as_supermario_event_read(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mario_query: Query<(Entity, &mut Transform), With<Mario>>,
    mut mario_changed_event_reader: EventReader<MarioChangedAsSuperMarioEvent>,
    texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    if mario_query.is_empty() || mario_changed_event_reader.is_empty() {
        return;
    }

    let mario = mario_query.single();
    if let Some(event) = mario_changed_event_reader.iter().next() {
        let x = event.x;
        let y = event.y + 7.0;
        commands.entity(mario.0).despawn();
        commands.spawn(get_mario_bundle(asset_server, texture_atlases, true, x, y));
    }
}

fn mario_move_event_read(
    mut mario_move_event_reader: EventReader<MarioMoveEvent>,
    mut magic_mushroom_query: Query<(&mut Visibility, &Transform), With<MagicMushroom>>,
    mut mario_changed_event_writer: EventWriter<MarioChangedAsSuperMarioEvent>,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
) {
    for event in mario_move_event_reader.iter() {
        for (mut visibility, transform) in magic_mushroom_query.iter_mut() {
            if visibility.is_visible
                && event.x >= transform.translation.x - 10.0
                && event.x <= transform.translation.x + 10.0
            {
                let powerup = asset_server.load("powerup.ogg");
                audio.play(powerup);
                visibility.is_visible = false;
                mario_changed_event_writer.send(MarioChangedAsSuperMarioEvent {
                    x: event.x,
                    y: event.y,
                });
            }
        }
    }
}

fn enemy_dead_event_read(
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

fn enemy_move_event_read_for_mario(
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

fn hit_questionblock_by_mario(
    commands: Commands,
    mario_query: Query<&Transform, With<Mario>>,
    question_block_query: Query<(Entity, &Transform, &QuestionBlock), With<QuestionBlock>>,
    empty_block_query: Query<(&mut Visibility, &Transform), With<EmptyBlock>>,
    magic_mushroom_event_writer: EventWriter<MagicMushroomReleaseEvent>,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
) {
    if mario_query.is_empty() {
        return;
    }

    let transform = mario_query.single();

    handle_hit_questionblock(
        commands,
        transform,
        question_block_query,
        empty_block_query,
        magic_mushroom_event_writer,
        asset_server,
        audio,
    );
}

fn handle_hit_questionblock(
    mut commands: Commands,
    mario_transform: &Transform,
    question_block_query: Query<(Entity, &Transform, &QuestionBlock), With<QuestionBlock>>,
    mut empty_block_query: Query<(&mut Visibility, &Transform), With<EmptyBlock>>,
    mut magic_mushroom_event_writer: EventWriter<MagicMushroomReleaseEvent>,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
) {
    for (ent, question_block_transform, question_block) in question_block_query.iter() {
        if (mario_transform.translation.x >= question_block_transform.translation.x - 5.0
            && mario_transform.translation.x <= question_block_transform.translation.x + 5.0)
            && (mario_transform.translation.y >= question_block_transform.translation.y - 5.0
                && mario_transform.translation.y <= question_block_transform.translation.y + 5.0)
        {
            commands.entity(ent).despawn();

            for (mut empty_block_visibility, empty_block_transform) in empty_block_query.iter_mut()
            {
                if question_block_transform.translation.x == empty_block_transform.translation.x
                    && question_block_transform.translation.y == empty_block_transform.translation.y
                {
                    empty_block_visibility.is_visible = true;

                    if question_block.is_mushroom {
                        let mushroom_appears = asset_server.load("mushroom_appears.ogg");
                        audio.play(mushroom_appears);
                        magic_mushroom_event_writer.send(MagicMushroomReleaseEvent {
                            x: empty_block_transform.translation.x,
                            y: empty_block_transform.translation.y,
                        });
                    } else {
                        let coin_sound = asset_server.load("coin.ogg");
                        audio.play(coin_sound);
                    }
                }
            }
        }
    }
}

fn move_enemy(
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

fn handle_mario_dead_event(
    mut mario_query: Query<(&mut Transform, &mut AnimationTimer, &mut MarioDead), With<MarioDead>>,
) {
    if mario_query.is_empty() {
        return;
    }

    for (mut transform, mut animation_timer, mut mario) in mario_query.iter_mut() {
        animation_timer.tick(Duration::from_secs_f32(1.0));
        if animation_timer.just_finished() {
            if mario.go_up {
                transform.translation.y += 2.0;
                if transform.translation.y >= -30.0 {
                    mario.go_up = false;
                }
            } else {
                transform.translation.y -= 1.5;
            }
        }
    }
}

fn move_mario(
    time: Res<Time>,
    audio: Res<Audio>,
    asset_server: Res<AssetServer>,
    keyboard_input: Res<Input<KeyCode>>,
    mut mario_query: Query<
        (
            &AnimationIndices,
            &mut Transform,
            &mut TextureAtlasSprite,
            &mut AnimationTimer,
            &mut Mario,
        ),
        With<Mario>,
    >,
    mut mario_move_event_writer: EventWriter<MarioMoveEvent>,
) {
    if mario_query.is_empty() {
        return;
    }

    for (
        animation_indices,
        mut transform,
        mut texture_atlas_sprite,
        mut animation_timer,
        mut mario,
    ) in mario_query.iter_mut()
    {
        let mut direction_x = 0.0;
        let mut direction_y = 0.0;

        let min_y = if !mario.is_super_mario { -78.0 } else { -70.0 };
        let max_y = -10.0;

        if keyboard_input.pressed(KeyCode::Left) {
            animation_timer.tick(time.delta());
            direction_x -= 1.2;
            texture_atlas_sprite.flip_x = true;

            if animation_timer.just_finished() && !keyboard_input.pressed(KeyCode::Up) {
                texture_atlas_sprite.index = if texture_atlas_sprite.index == animation_indices.last
                {
                    animation_indices.first
                } else {
                    texture_atlas_sprite.index + 1
                };
            }
        }

        if keyboard_input.pressed(KeyCode::Right) {
            animation_timer.tick(time.delta());
            direction_x += 1.2;
            texture_atlas_sprite.flip_x = false;

            if animation_timer.just_finished() && !keyboard_input.pressed(KeyCode::Up) {
                texture_atlas_sprite.index = if texture_atlas_sprite.index == animation_indices.last
                {
                    animation_indices.first
                } else {
                    texture_atlas_sprite.index + 1
                };
            }
        }

        if keyboard_input.pressed(KeyCode::Up) {
            animation_timer.tick(Duration::from_secs_f32(1.0));

            if animation_timer.just_finished() {
                if transform.translation.y == min_y {
                    let mario_jump_audio = if !mario.is_super_mario {
                        asset_server.load("mario_jump.ogg")
                    } else {
                        asset_server.load("super_mario_jump.ogg")
                    };

                    audio.play(mario_jump_audio);
                }
                if !mario.dont_go_up_until_settle {
                    direction_y += 1.5;
                } else {
                    direction_y -= 1.5;
                }
            }
        } else {
            direction_y -= 1.5;
        }

        let mario_position_x = transform.translation.x + direction_x;
        let mario_position_y = transform.translation.y + direction_y;

        transform.translation.x = mario_position_x.clamp(-250.0, 180.0);
        transform.translation.y = mario_position_y.clamp(min_y, max_y);

        if transform.translation.y == max_y {
            mario.dont_go_up_until_settle = true;
        } else if transform.translation.y == min_y {
            mario.dont_go_up_until_settle = false;
        }

        if direction_x != 0.0 {
            mario_move_event_writer.send(MarioMoveEvent {
                x: transform.translation.x,
                y: transform.translation.y,
            });
        }
    }
}
