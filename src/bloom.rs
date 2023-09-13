//! Illustrates bloom post-processing in 2d.

use bevy::prelude::*;

use crate::systems::{setup_bloom, update_bloom_settings};

pub fn init_bloom_animation() {
    App::new()
        .insert_resource(ClearColor(Color::DARK_GRAY))
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup_bloom)
        .add_systems(Update, update_bloom_settings)
        .run();
}

// ------------------------------------------------------------------------------------------------
