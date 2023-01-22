use bevy::{prelude::{Commands, Res, Query, Visibility, Entity, With, Component, AssetServer, Audio}, time::Time, sprite::TextureAtlasSprite};

use crate::{AnimationTimer, animation::{AnimationIndices, animate}};

#[derive(Component)]
pub struct Firework;

pub fn show_fireworks(
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

pub fn animate_fireworks(
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