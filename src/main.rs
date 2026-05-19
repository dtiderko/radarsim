use bevy::{
    input::common_conditions::input_just_pressed,
    prelude::*,
    sprite_render::{Wireframe2dConfig, Wireframe2dPlugin},
};
use bevy_egui::prelude::*;

use crate::{common::*, tweaks::Tweaks};

mod common;
mod radar_scene;
mod real_scene;
mod tweaks;

fn main() {
    App::new()
        .insert_resource(SimTime(0.))
        .add_plugins(DefaultPlugins)
        .add_plugins(Wireframe2dPlugin::default())
        .add_plugins(EguiPlugin::default())
        .add_plugins(tweaks::TweaksUi)
        .add_plugins(real_scene::RealScene)
        .add_plugins(radar_scene::RadarScene)
        .add_systems(Startup, setup_camera_system)
        .add_systems(
            Update,
            toggle_wireframe.run_if(input_just_pressed(KeyCode::Space)),
        )
        .add_systems(Update, advance_sim_time)
        .run();
}

fn toggle_wireframe(mut wireframe_config: ResMut<Wireframe2dConfig>) {
    wireframe_config.global = !wireframe_config.global;
}

fn setup_camera_system(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn advance_sim_time(time: Res<Time>, tweaks: Res<Tweaks>, mut sim_time: ResMut<SimTime>) {
    sim_time.0 += time.delta_secs() * tweaks.time_scale;
}
