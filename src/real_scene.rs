use bevy::prelude::*;
use nalgebra::vector;

use crate::{common::*, tweaks::Tweaks};

pub struct RealScene;
impl Plugin for RealScene {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_aircraft)
            .add_systems(Startup, setup_sensor)
            .add_systems(Update, (move_aircraft, update_render).chain())
            .add_systems(Update, draw_arrow);
    }
}

fn setup_aircraft(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let shape = meshes.add(Circle::new(100.0));
    let color = Color::srgb(1., 0., 0.);
    let pos = Position(vector![0., 0.]);

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
    let shape = meshes.add(Circle::new(100.0));
    let color = Color::srgb(0., 1., 0.);
    let pos = Position(vector![5000., 5000.]);

    commands.spawn((
        Sensor,
        // render
        Mesh2d(shape),
        MeshMaterial2d(materials.add(color)),
        Transform::from_xyz(pos.x, pos.y, 0.),
        // data
        pos,
    ));
}

fn move_aircraft(
    sim_time: Res<SimTime>,
    mut query: Query<(&mut Position, &mut Velocity, &mut Acceleration), With<Aircraft>>,
) {
    const V: f32 = 300.; // m/s
    const Q: f32 = 9.; // m/s^2

    let a = V.powi(2) / Q;
    let w = Q / (2. * V);

    for (mut pos, mut vel, mut acc) in &mut query {
        pos.0 = vector![
            a * f32::sin(w * sim_time.0),
            a * f32::sin(2. * w * sim_time.0),
        ];

        vel.0 = vector![
            V * (f32::cos(w * sim_time.0) / 2.),
            V * f32::cos(2. * w * sim_time.0),
        ];

        acc.0 = vector![
            -Q * (f32::sin(w * sim_time.0) / 4.),
            -Q * f32::sin(2. * w * sim_time.0),
        ];
    }
}

fn update_render(mut query: Query<(&Position, &mut Transform), With<Aircraft>>) {
    for (sim_pos, mut render_pos) in &mut query {
        render_pos.translation[0] = sim_pos.x;
        render_pos.translation[1] = sim_pos.y;
    }
}

fn draw_arrow(
    mut gizmos: Gizmos,
    aircraft: Single<(&Position, &Velocity), With<Aircraft>>,
    tweaks: Res<Tweaks>,
) {
    gizmos.arrow_2d(
        Vec2::new(aircraft.0.x, aircraft.0.y),
        Vec2::new(
            aircraft.0.x + aircraft.1.x * tweaks.arrow_scale,
            aircraft.0.y + aircraft.1.y * tweaks.arrow_scale,
        ),
        Color::srgb(1., 0., 0.),
    );
}
