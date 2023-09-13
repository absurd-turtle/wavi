use std::sync::{atomic::AtomicU8, Arc};

use bevy::prelude::Resource;

#[derive(Resource)]
pub struct Beats {
    pub detect_count: Arc<AtomicU8>,
    pub react_count: Arc<AtomicU8>,
}
