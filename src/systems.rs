use std::{
    io::{self, BufRead},
    process::Command,
    thread,
};

use bevy::{
    core_pipeline::{
        bloom::{BloomCompositeMode, BloomSettings},
        tonemapping::Tonemapping,
    },
    prelude::{
        default, shape, AssetServer, Assets, Camera, Camera2dBundle, Color, Commands, Entity,
        Input, KeyCode, Mesh, Query, Res, ResMut, TextBundle, Transform, Vec2, Vec3, With,
    },
    sprite::{ColorMaterial, MaterialMesh2dBundle, Sprite, SpriteBundle},
    text::{Text, TextStyle},
    time::Time,
    ui::{PositionType, Style, Val},
};
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

fn increment_react_count(beats: Res<Beats>) {
    let react = beats.react_count.load(std::sync::atomic::Ordering::Relaxed);
    if react == 255 {
        beats
            .react_count
            .store(0, std::sync::atomic::Ordering::Relaxed);
    } else {
        beats
            .react_count
            .store(react + 1, std::sync::atomic::Ordering::Relaxed);
    }
}

fn check_beat(beats: &Res<Beats>) -> bool {
    let detect = beats
        .detect_count
        .load(std::sync::atomic::Ordering::Relaxed);
    let react = beats.react_count.load(std::sync::atomic::Ordering::Relaxed);
    println!("detect: {}", detect);
    println!("react: {}", react);
    println!("{}", detect != react);
    detect != react
}

pub fn transform_anim_on_beat(beats: Res<Beats>, shape_query: Query<&mut Transform, With<Shape>>) {
    if check_beat(&beats) {
        increment_react_count(beats);

        println!("do something");
        shape_anim_transform(shape_query);
    }
}

// BLOOM
// ---------------------------------------------------
pub fn setup_bloom(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                hdr: true, // 1. HDR is required for bloom
                ..default()
            },
            tonemapping: Tonemapping::TonyMcMapface, // 2. Using a tonemapper that desaturates to white is recommended
            ..default()
        },
        BloomSettings::default(), // 3. Enable bloom for the camera
    ));

    // Sprite
    commands.spawn(SpriteBundle {
        texture: asset_server.load("branding/icon.png"),
        sprite: Sprite {
            color: Color::rgb(5.0, 5.0, 5.0), // 4. Put something bright in a dark environment to see the effect
            custom_size: Some(Vec2::splat(160.0)),
            ..default()
        },
        ..default()
    });

    // Circle mesh
    commands.spawn(MaterialMesh2dBundle {
        mesh: meshes.add(shape::Circle::new(100.).into()).into(),
        // 4. Put something bright in a dark environment to see the effect
        material: materials.add(ColorMaterial::from(Color::rgb(7.5, 0.0, 7.5))),
        transform: Transform::from_translation(Vec3::new(-200., 0., 0.)),
        ..default()
    });

    // Hexagon mesh
    commands.spawn(MaterialMesh2dBundle {
        mesh: meshes
            .add(shape::RegularPolygon::new(100., 6).into())
            .into(),
        // 4. Put something bright in a dark environment to see the effect
        material: materials.add(ColorMaterial::from(Color::rgb(6.25, 9.4, 9.1))),
        transform: Transform::from_translation(Vec3::new(200., 0., 0.)),
        ..default()
    });

    // UI
    commands.spawn(
        TextBundle::from_section(
            "",
            TextStyle {
                font_size: 18.0,
                color: Color::WHITE,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        }),
    );
}

pub fn update_bloom_settings(
    mut camera: Query<(Entity, Option<&mut BloomSettings>), With<Camera>>,
    mut text: Query<&mut Text>,
    mut commands: Commands,
    keycode: Res<Input<KeyCode>>,
    time: Res<Time>,
    beats: Res<Beats>,
) {
    let bloom_settings = camera.single_mut();
    let mut text = text.single_mut();
    let text = &mut text.sections[0].value;

    match bloom_settings {
        (entity, Some(mut bloom_settings)) => {
            *text = "BloomSettings (Toggle: Space)\n".to_string();
            text.push_str(&format!("(Q/A) Intensity: {}\n", bloom_settings.intensity));
            text.push_str(&format!(
                "(W/S) Low-frequency boost: {}\n",
                bloom_settings.low_frequency_boost
            ));
            text.push_str(&format!(
                "(E/D) Low-frequency boost curvature: {}\n",
                bloom_settings.low_frequency_boost_curvature
            ));
            text.push_str(&format!(
                "(R/F) High-pass frequency: {}\n",
                bloom_settings.high_pass_frequency
            ));
            text.push_str(&format!(
                "(T/G) Mode: {}\n",
                match bloom_settings.composite_mode {
                    BloomCompositeMode::EnergyConserving => "Energy-conserving",
                    BloomCompositeMode::Additive => "Additive",
                }
            ));
            text.push_str(&format!(
                "(Y/H) Threshold: {}\n",
                bloom_settings.prefilter_settings.threshold
            ));
            text.push_str(&format!(
                "(U/J) Threshold softness: {}\n",
                bloom_settings.prefilter_settings.threshold_softness
            ));

            if keycode.just_pressed(KeyCode::Space) {
                commands.entity(entity).remove::<BloomSettings>();
            }

            let dt = time.delta_seconds();

            if check_beat(&beats) {
                let new = bloom_settings.intensity + 0.1;
                bloom_settings.intensity = if new > 0.3 { 0.3 } else { new };
                println!("{}", dt);
                println!("intensity: {}", bloom_settings.intensity);
                increment_react_count(beats);
            } else {
                bloom_settings.intensity -= dt / 4.0;
            }

            if keycode.pressed(KeyCode::A) {
                bloom_settings.intensity -= dt / 10.0;
            }
            if keycode.pressed(KeyCode::Q) {
                bloom_settings.intensity += dt / 10.0;
            }
            bloom_settings.intensity = bloom_settings.intensity.clamp(0.0, 1.0);

            if keycode.pressed(KeyCode::S) {
                bloom_settings.low_frequency_boost -= dt / 10.0;
            }
            if keycode.pressed(KeyCode::W) {
                bloom_settings.low_frequency_boost += dt / 10.0;
            }
            bloom_settings.low_frequency_boost = bloom_settings.low_frequency_boost.clamp(0.0, 1.0);

            if keycode.pressed(KeyCode::D) {
                bloom_settings.low_frequency_boost_curvature -= dt / 10.0;
            }
            if keycode.pressed(KeyCode::E) {
                bloom_settings.low_frequency_boost_curvature += dt / 10.0;
            }
            bloom_settings.low_frequency_boost_curvature =
                bloom_settings.low_frequency_boost_curvature.clamp(0.0, 1.0);

            if keycode.pressed(KeyCode::F) {
                bloom_settings.high_pass_frequency -= dt / 10.0;
            }
            if keycode.pressed(KeyCode::R) {
                bloom_settings.high_pass_frequency += dt / 10.0;
            }
            bloom_settings.high_pass_frequency = bloom_settings.high_pass_frequency.clamp(0.0, 1.0);

            if keycode.pressed(KeyCode::G) {
                bloom_settings.composite_mode = BloomCompositeMode::Additive;
            }
            if keycode.pressed(KeyCode::T) {
                bloom_settings.composite_mode = BloomCompositeMode::EnergyConserving;
            }

            if keycode.pressed(KeyCode::H) {
                bloom_settings.prefilter_settings.threshold -= dt;
            }
            if keycode.pressed(KeyCode::Y) {
                bloom_settings.prefilter_settings.threshold += dt;
            }
            bloom_settings.prefilter_settings.threshold =
                bloom_settings.prefilter_settings.threshold.max(0.0);

            if keycode.pressed(KeyCode::J) {
                bloom_settings.prefilter_settings.threshold_softness -= dt / 10.0;
            }
            if keycode.pressed(KeyCode::U) {
                bloom_settings.prefilter_settings.threshold_softness += dt / 10.0;
            }
            bloom_settings.prefilter_settings.threshold_softness = bloom_settings
                .prefilter_settings
                .threshold_softness
                .clamp(0.0, 1.0);
        }

        (entity, None) => {
            *text = "Bloom: Off (Toggle: Space)".to_string();

            if keycode.just_pressed(KeyCode::Space) {
                commands.entity(entity).insert(BloomSettings::default());
            }
        }
    }
}
// ---------------------------------------------------
