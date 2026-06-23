use bevy::{
    prelude::*,
    sprite_render::{Material2dPlugin, Wireframe2dPlugin},
};
use bevy_egui::prelude::*;

use crate::{camera::*, common::*, normal_dist_material::NormalDistMaterial, tweaks::Tweaks};

mod camera;
mod chi_squared;
mod common;
mod kalman_scene;
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
        .add_plugins(kalman_scene::KalmanScene)
        .add_systems(Update, advance_sim_time)
        .add_systems(Startup, setup_camera)
        .add_systems(
            Update,
            autozoom_camera.run_if(|tweaks: Res<Tweaks>| tweaks.auto_camera),
        )
        .add_systems(
            Update,
            (pan_camera, zoom_camera).run_if(|tweaks: Res<Tweaks>| !tweaks.auto_camera),
        )
        .run();
}

fn advance_sim_time(time: Res<Time>, tweaks: Res<Tweaks>, mut sim_time: ResMut<SimTime>) {
    sim_time.0 += time.delta_secs() * tweaks.time_scale;
}
