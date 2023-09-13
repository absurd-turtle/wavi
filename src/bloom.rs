//! Illustrates bloom post-processing in 2d.

use std::sync::{atomic::AtomicU8, Arc};

use bevy::prelude::*;

use crate::{
    resources::Beats,
    systems::{init_beat_tracker, setup_bloom, update_bloom_settings},
};

pub fn init_bloom_animation() {
    App::new()
        .insert_resource(ClearColor(Color::DARK_GRAY))
        .add_plugins((DefaultPlugins, BloomPlugin, BeatPlugin))
        .run();
}

pub struct BloomPlugin;

impl Plugin for BloomPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_bloom);
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
        app.add_systems(Update, update_bloom_settings);
    }
}

// ------------------------------------------------------------------------------------------------
