use std::sync::{atomic::AtomicU8, Arc};

use bevy::prelude::*;

use crate::{
    components::Shape,
    resources::Beats,
    systems::{init_beat_tracker, read_beat},
};

pub fn init_rectangle_animation() {
    App::new()
        .add_plugins((DefaultPlugins, ShapePlugin, BeatPlugin))
        .run();
}

pub fn setup_shapes(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    // Rectangle
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.25, 0.25, 0.75),
                custom_size: Some(Vec2::new(50.0, 100.0)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(-50., 0., 0.)),
            ..default()
        },
        Shape {},
    ));
}

pub struct ShapePlugin;

impl Plugin for ShapePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_shapes);
    }
}

pub struct BeatPlugin;

impl Plugin for BeatPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Beats {
            detect_count: Arc::new(AtomicU8::new(0)),
            react_count: Arc::new(AtomicU8::new(0)),
        });
        app.add_systems(Startup, init_beat_tracker);
        app.add_systems(Update, read_beat);
    }
}
