use bevy::{
    input::common_conditions::input_just_pressed,
    prelude::*,
    sprite_render::{Wireframe2dConfig, Wireframe2dPlugin},
};
use bevy_egui::prelude::*;

mod common;
mod radar_scene;
mod real_scene;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(Wireframe2dPlugin::default())
        .add_plugins(EguiPlugin::default())
        .add_plugins(real_scene::RealScene)
        .add_plugins(radar_scene::RadarScene)
        .add_systems(Startup, setup_camera_system)
        .add_systems(
            Update,
            toggle_wireframe.run_if(input_just_pressed(KeyCode::Space)),
        )
        .run();
}

fn toggle_wireframe(mut wireframe_config: ResMut<Wireframe2dConfig>) {
    wireframe_config.global = !wireframe_config.global;
}

fn setup_camera_system(mut commands: Commands) {
    commands.spawn(Camera2d);
}
