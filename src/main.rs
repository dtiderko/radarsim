use bevy::{
    prelude::*,
    sprite_render::{Material2dPlugin, Wireframe2dPlugin},
};
use bevy_egui::prelude::*;

use crate::{common::*, normal_dist_material::NormalDistMaterial, tweaks::Tweaks};

mod common;
mod normal_dist_material;
mod radar_scene;
mod real_scene;
mod tweaks;

fn main() {
    App::new()
        .insert_resource(SimTime(0.))
        .add_plugins(DefaultPlugins)
        .add_plugins(Material2dPlugin::<NormalDistMaterial>::default())
        .add_plugins(Wireframe2dPlugin::default())
        .add_plugins(EguiPlugin::default())
        .add_plugins(tweaks::TweaksUi)
        .add_plugins(real_scene::RealScene)
        .add_plugins(radar_scene::RadarScene)
        .add_systems(Startup, setup_camera_system)
        .add_systems(Update, advance_sim_time)
        .add_systems(Update, zoom_camera)
        .run();
}

fn setup_camera_system(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn advance_sim_time(time: Res<Time>, tweaks: Res<Tweaks>, mut sim_time: ResMut<SimTime>) {
    sim_time.0 += time.delta_secs() * tweaks.time_scale;
}

fn zoom_camera(
    entities: Query<&Transform, (With<Mesh2d>, Without<Camera2d>)>,
    camera: Single<(&mut Transform, &mut Projection), With<Camera2d>>,
    window: Single<&Window>,
) {
    const PADDING: f32 = 1.5;

    let (mut pos, mut proj) = camera.into_inner();

    // figure out the area in which all entities are
    let min = entities.iter().fold(Vec2::splat(f32::MAX), |acc, e| {
        acc.min(e.translation.truncate())
    });
    let max = entities.iter().fold(Vec2::splat(f32::MIN), |acc, e| {
        acc.max(e.translation.truncate())
    });

    // position camera in the center of all entities
    let center = (min + max) * 0.5;
    pos.translation.x = center.x;
    pos.translation.y = center.y;

    let window_size = Vec2::new(window.width(), window.height());

    let size = max - min;
    let x_ratio = size.x / window_size.x;
    let y_ratio = size.y / window_size.y;

    if let Projection::Orthographic(orth_proj) = &mut *proj {
        orth_proj.scale = f32::max(x_ratio, y_ratio) * PADDING;
    }
}
