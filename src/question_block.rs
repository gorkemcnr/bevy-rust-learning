use bevy::{prelude::{Res, Component, Query, With, Commands, Transform, Entity, Visibility, EventWriter, AssetServer, Audio}, time::Time, sprite::TextureAtlasSprite};

use crate::{AnimationTimer, animation::{AnimationIndices, animate}, mario::Mario, mushroom::MagicMushroomReleaseEvent};

#[derive(Component)]
pub struct QuestionBlock {
    pub is_mushroom: bool,
}

#[derive(Component)]
pub struct EmptyBlock;

pub fn animate_question_blocks(
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


pub fn hit_questionblock_by_mario(
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

pub fn handle_hit_questionblock(
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