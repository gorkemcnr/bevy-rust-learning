use std::time::Duration;

use bevy::prelude::{Query, Transform, Visibility, With, Component, EventReader};

use crate::animation::AnimationTimer;

#[derive(Component)]
pub struct MagicMushroom {
    pub is_released: bool,
    pub x_reached_max: bool,
}

pub struct MagicMushroomReleaseEvent {
    pub x: f32,
    pub y: f32,
}

pub fn move_mushroom(
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

pub fn magic_mushroom_event_read(
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
