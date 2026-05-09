use bevy::{
    input::common_conditions::input_just_pressed,
    prelude::*,
    sprite_render::{Wireframe2dConfig, Wireframe2dPlugin},
};
use bevy_egui::prelude::*;

#[derive(Resource)]
struct SimTime(f32);

#[derive(Component)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Component)]
struct Aircraft;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(Wireframe2dPlugin::default())
        .add_systems(
            Update,
            toggle_wireframe.run_if(input_just_pressed(KeyCode::Space)),
        )
        .add_plugins(EguiPlugin::default())
        .add_systems(Startup, setup_camera_system)
        .add_systems(Startup, setup_aircraft)
        .add_systems(Update, update_aircraft)
        .run();
}

fn toggle_wireframe(mut wireframe_config: ResMut<Wireframe2dConfig>) {
    wireframe_config.global = !wireframe_config.global;
}

fn setup_camera_system(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn setup_aircraft(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.insert_resource(SimTime(0.));

    let shape = meshes.add(Circle::new(5.0));
    let color = Color::srgb(1., 0., 0.);
    let pos = Position { x: 0.0, y: 0.0 };

    commands.spawn((
        Aircraft,
        // render
        Mesh2d(shape),
        MeshMaterial2d(materials.add(color)),
        Transform::from_xyz(pos.x, pos.y, 0.),
        // data
        pos,
    ));
}

fn update_aircraft(
    time: Res<Time>,
    mut sim_time: ResMut<SimTime>,
    mut query: Query<(&mut Position, &mut Transform), With<Aircraft>>,
) {
    const DISP_SCALE: f32 = 0.01;
    const TIME_SCALE: f32 = 100.;

    const V: f32 = 300.;
    const Q: f32 = 9.;

    let a = V.powi(2) / Q;
    let w = Q / (2. * V);

    sim_time.0 += time.delta_secs() * TIME_SCALE;

    for (mut spos, mut rpos) in &mut query {
        spos.x = a * f32::sin(w * sim_time.0);
        spos.y = a * f32::sin(2. * w * sim_time.0);

        rpos.translation[0] = spos.x * DISP_SCALE;
        rpos.translation[1] = spos.y * DISP_SCALE;
    }
}
