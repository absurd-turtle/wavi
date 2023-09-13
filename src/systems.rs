use std::{
    io::{self, BufRead},
    process::Command,
    thread,
};

use bevy::prelude::{Query, Res, Transform, With};
use rand::Rng;

use crate::{components::Shape, resources::Beats};

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

/** get shape and rotate and scale it randomly */
fn shape_anim_transform(mut shape_query: Query<&mut Transform, With<Shape>>) {
    if let Ok(mut shape) = shape_query.get_single_mut() {
        let mut rng = rand::thread_rng();
        let rotation: f32 = rng.gen_range(-0.4..0.4);
        shape.rotate_z(rotation);
        let scale: f32 = rng.gen_range(0.0..2.0);
        if shape.scale.x > 2.0 {
            shape.scale -= scale;
        } else {
            shape.scale += scale;
        }
        println!("scale: {}", shape.scale);
    }
}

pub fn read_beat(beats: Res<Beats>, mut shape_query: Query<&mut Transform, With<Shape>>) {
    let detect = beats
        .detect_count
        .load(std::sync::atomic::Ordering::Relaxed);
    let react = beats.react_count.load(std::sync::atomic::Ordering::Relaxed);
    println!("{}", detect);
    if detect != react {
        if react == 255 {
            beats
                .react_count
                .store(0, std::sync::atomic::Ordering::Relaxed);
        } else {
            beats
                .react_count
                .store(react + 1, std::sync::atomic::Ordering::Relaxed);
        }

        println!("do something");
        shape_anim_transform(shape_query);
    }
}
