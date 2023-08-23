use bevy::prelude::*;
use rand::prelude::*;
use std::{
    io::{self, BufRead},
    process::Command,
    sync::{atomic::AtomicU8, Arc},
    thread,
};

#[derive(Resource)]
pub struct Beats {
    detect_count: Arc<AtomicU8>,
    react_count: Arc<AtomicU8>,
}

#[derive(Component)]
pub struct Shape {}

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

pub fn init_beat_tracker(beats: Res<Beats>) {
    let detect_count = beats.detect_count.clone();
    // Spawn a new thread to run the child process
    thread::spawn(move || {
        // Command to run the external program
        let mut cmd = Command::new("DBNBeatTracker");
        cmd.arg("online");

        // Redirect stdout to a pipe to capture the output
        cmd.stdout(std::process::Stdio::piped());

        // Run the external program and capture its stdout
        let mut child = cmd.spawn().expect("Failed to start the external program");

        // Get a handle to the stdout of the external program
        let stdout = child.stdout.take().expect("Failed to capture stdout");

        // Read and send stdout data line by line
        let reader = io::BufReader::new(stdout);
        for line in reader.lines() {
            if let Ok(line) = line {
                println!("beat detected at: {}", line);
                let current = detect_count
                    .clone()
                    .load(std::sync::atomic::Ordering::Relaxed);
                if current == 255 {
                    println!("set detect_count to 0");
                    detect_count
                        .clone()
                        .store(0, std::sync::atomic::Ordering::Relaxed);
                } else {
                    println!("increment detect_count");
                    detect_count
                        .clone()
                        .store(current + 1, std::sync::atomic::Ordering::Relaxed);
                }
            }
        }
    });
}

pub fn read_beat(beats: Res<Beats>, mut shape_query: Query<&mut Transform, With<Shape>>) {
    let detect = beats
        .detect_count
        .load(std::sync::atomic::Ordering::Relaxed);
    let react = beats.react_count.load(std::sync::atomic::Ordering::Relaxed);
    println!("{}", detect);
    if detect != react {
        if react == 255 {
            beats.react_count
                .store(0, std::sync::atomic::Ordering::Relaxed);
        } else {
            beats.react_count
                .store(react + 1, std::sync::atomic::Ordering::Relaxed);
        }

        println!("do something");
        if let Ok(mut transform) = shape_query.get_single_mut() {
            let mut rng = rand::thread_rng();
            let rotation: f32 = rng.gen_range(-0.4..0.4);
            transform.rotate_z(rotation);
            let scale: f32 = rng.gen_range(0.0..2.0);
            if transform.scale.x > 2.0 {
                transform.scale -= scale;
            } else {
                transform.scale += scale;
            }
            println!("scale: {}", transform.scale);
        }
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

pub struct ShapePlugin;

impl Plugin for ShapePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_shapes);
    }
}
