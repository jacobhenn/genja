#![feature(clamp_magnitude)]

mod oscilloscope_audio;

use std::{f32::consts::TAU, iter};

use bevy::{
    camera_controller::free_camera::{FreeCamera, FreeCameraPlugin, FreeCameraState},
    color::palettes::tailwind,
    prelude::*,
};
use bevy_polyline::prelude::*;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugins(PolylinePlugin)
        .add_plugins(FreeCameraPlugin)
        .add_plugins((CameraPlugin, CameraSettingsPlugin, ScenePlugin))
        .add_plugins(oscilloscope_audio::AudioPlugin);
    oscilloscope_audio::setup_audio(&mut app);
    app.run();
}

struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera);
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 5.0).looking_at(Vec3::ZERO, Dir3::Y),
        AmbientLight { color: Color::WHITE, brightness: 800.0, ..default() },
        FreeCamera {
            sensitivity: 0.8,
            friction: 25.0,
            walk_speed: 3.0,
            run_speed: 9.0,
            ..default()
        },
    ));
}

struct CameraSettingsPlugin;
impl Plugin for CameraSettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_camera_settings);
    }
}

fn update_camera_settings(
    mut camera_query: Query<(&mut FreeCamera, &mut FreeCameraState)>,
    input: Res<ButtonInput<KeyCode>>,
) {
    let (mut free_camera, mut free_camera_state) = camera_query.single_mut().unwrap();

    if input.pressed(KeyCode::KeyZ) {
        free_camera.sensitivity = (free_camera.sensitivity - 0.005).max(0.005);
    }
    if input.pressed(KeyCode::KeyX) {
        free_camera.sensitivity += 0.005;
    }
    if input.pressed(KeyCode::KeyC) {
        free_camera.friction = (free_camera.friction - 0.2).max(0.0);
    }
    if input.pressed(KeyCode::KeyV) {
        free_camera.friction += 0.2;
    }
    if input.pressed(KeyCode::KeyF) {
        free_camera.scroll_factor = (free_camera.scroll_factor - 0.02).max(0.02);
    }
    if input.pressed(KeyCode::KeyG) {
        free_camera.scroll_factor += 0.02;
    }
    if input.just_pressed(KeyCode::KeyB) {
        free_camera_state.enabled = !free_camera_state.enabled;
    }
}

#[derive(Resource)]
struct ParametricPath {
    f: fn(f32) -> Vec3,
}

fn default_path(t: f32) -> Vec3 {
    Vec3::new((t * 4.0 * TAU).cos(), (t * 4.0 * TAU).sin(), t * 2.0)
}

struct ScenePlugin;
impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ParametricPath { f: default_path });
        app.add_systems(Startup, (spawn_axes, spawn_path));
    }
}

fn spawn_axes(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // let axes = [Dir3::X, Dir3::Y, Dir3::Z]
    //     .map(|dir| meshes.add(Segment3d::from_direction_and_length(dir, 1.0e3)));

    // let materials = [tailwind::RED_500, tailwind::GREEN_500, tailwind::BLUE_500]
    //     .map(|col| materials.add(Color::from(col)));

    // for (axis, material) in iter::zip(axes, materials) {
    //     commands.spawn((Mesh3d(axis.clone()), MeshMaterial3d(material.clone())));
    // }
}

// fn spawn_axes(
//     mut commands: Commands,
//     mut polyline_materials: ResMut<Assets<PolylineMaterial>>,
//     mut polylines: ResMut<Assets<Polyline>>,
// ) {
//     let axes = [Vec3::X, Vec3::Y, Vec3::Z]
//         .map(|u| polylines.add(Polyline { vertices: (-10..=10).map(|a| u * a as f32).collect() }));

//     let materials = [tailwind::RED_500, tailwind::GREEN_500, tailwind::BLUE_500].map(|col| {
//         polyline_materials.add(PolylineMaterial {
//             width: 1.0,
//             color: col.into(),
//             perspective: false,
//             ..default()
//         })
//     });

//     for (axis, material) in iter::zip(axes, materials) {
//         commands.spawn(PolylineBundle {
//             polyline: PolylineHandle(axis),
//             material: PolylineMaterialHandle(material),
//             ..default()
//         });
//     }
// }

fn spawn_path(
    mut commands: Commands,
    mut polyline_materials: ResMut<Assets<PolylineMaterial>>,
    mut polylines: ResMut<Assets<Polyline>>,
    path: Res<ParametricPath>,
) {
    let samples = 1000;
    let polyline_vertices = (0..samples).map(|i| (path.f)(i as f32 / samples as f32));
    let polyline = polylines.add(Polyline { vertices: polyline_vertices.collect() });
    let polyline_material = polyline_materials.add(PolylineMaterial {
        width: 3.0,
        color: tailwind::RED_500.into(),
        perspective: false,
        ..default()
    });

    commands.spawn(PolylineBundle {
        polyline: PolylineHandle(polyline),
        material: PolylineMaterialHandle(polyline_material),
        ..default()
    });
}
