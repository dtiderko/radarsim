use bevy::prelude::*;

use crate::common::*;

#[derive(Resource)]
struct SimTime(f32);

pub struct RealScene;
impl Plugin for RealScene {
    fn build(&self, app: &mut App) {
        app.insert_resource(SimTime(0.))
            .add_systems(Startup, setup_aircraft)
            .add_systems(Startup, setup_sensor)
            .add_systems(Update, (move_aircraft, update_render).chain());
    }
}

fn setup_aircraft(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
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
        Velocity::default(),
        Acceleration::default(),
    ));
}

fn setup_sensor(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let shape = meshes.add(Circle::new(5.0));
    let color = Color::srgb(0., 1., 0.);
    let pos = Position {
        x: 5000.0,
        y: 3000.0,
    };

    commands.spawn((
        Sensor,
        // render
        Mesh2d(shape),
        MeshMaterial2d(materials.add(color)),
        Transform::from_xyz(pos.x * DISP_SCALE, pos.y * DISP_SCALE, 0.),
        // data
        pos,
    ));
}

fn move_aircraft(
    time: Res<Time>,
    mut sim_time: ResMut<SimTime>,
    mut query: Query<(&mut Position, &mut Velocity, &mut Acceleration), With<Aircraft>>,
) {
    const V: f32 = 300.;
    const Q: f32 = 9.;

    let a = V.powi(2) / Q;
    let w = Q / (2. * V);

    sim_time.0 += time.delta_secs() * TIME_SCALE;

    for (mut pos, mut vel, mut acc) in &mut query {
        pos.x = a * f32::sin(w * sim_time.0);
        pos.y = a * f32::sin(2. * w * sim_time.0);

        vel.x = V * (f32::cos(w * sim_time.0) / 2.);
        vel.y = V * f32::cos(2. * w * sim_time.0);

        acc.x = -Q * (f32::sin(w * sim_time.0) / 4.);
        acc.y = -Q * f32::sin(2. * w * sim_time.0);
    }
}

fn update_render(mut query: Query<(&Position, &mut Transform), With<Aircraft>>) {
    for (sim_pos, mut render_pos) in &mut query {
        render_pos.translation[0] = sim_pos.x * DISP_SCALE;
        render_pos.translation[1] = sim_pos.y * DISP_SCALE;
    }
}
