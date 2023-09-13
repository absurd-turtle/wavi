use bloom::init_bloom_animation;
use rectangle::init_rectangle_animation;

mod bloom;
mod components;
mod rectangle;
mod resources;
mod systems;
mod utils;

fn main() {
    // init_rectangle_animation();
    init_bloom_animation();
}
