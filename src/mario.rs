use std::time::Duration;

use bevy::{prelude::{Component, Res, Audio, AssetServer, Input, KeyCode, Query, Transform, With, EventWriter, ResMut, Assets, Vec2, default, EventReader, Visibility, Commands, Entity}, time::{Time, TimerMode, Timer}, sprite::{TextureAtlasSprite, TextureAtlas, SpriteSheetBundle}};

use crate::{animation::{AnimationIndices, AnimationTimer}, mushroom::MagicMushroom};

#[derive(Component)]
pub struct MarioDead {
    go_up: bool,
}

#[derive(Component)]
pub struct Mario {
    dont_go_up_until_settle: bool,
    pub is_super_mario: bool,
}

pub struct MarioChangedAsSuperMarioEvent {
    x: f32,
    y: f32,
}

pub struct MarioMoveEvent {
    x: f32,
    y: f32,
}

pub fn move_mario(
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

pub fn handle_mario_dead_event(
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

pub fn get_dead_mario_bundle(
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

pub fn get_mario_bundle(
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

pub fn mario_changed_as_supermario_event_read(
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

pub fn mario_move_event_read(
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
