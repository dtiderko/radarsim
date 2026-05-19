use std::time::Duration;

use bevy::{prelude::*, time::common_conditions::on_timer};
use rand::prelude::*;

use crate::common::*;

#[derive(Component)]
struct CartesianMeasure;

#[derive(Component)]
struct PolarMeasure;

pub struct RadarScene;
impl Plugin for RadarScene {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (render_cartesian, render_polar)
                .run_if(on_timer(Duration::from_secs(5 / TIME_SCALE as u64))),
        );
    }
}

fn render_cartesian(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    query: Query<(&Position, &Velocity, &Acceleration), With<Aircraft>>,
) {
    const SIG: f32 = 50.;

    let shape = meshes.add(Circle::new(2.0));
    let color = Color::srgb(0., 1., 1.);
    let material = materials.add(color);

    let mut rng = rand::rng();

    let mut points = Vec::with_capacity(query.count());
    for (p, v, a) in query {
        points.push(Position {
            x: (p.x + v.x + a.x + SIG * rng.random_range(-1.0..=1.)),
            y: (p.y + v.y + a.y + SIG * rng.random_range(-1.0..=1.)),
        });
    }

    commands.spawn_batch(points.into_iter().map(move |p| {
        (
            CartesianMeasure,
            // render
            Mesh2d(shape.clone()),
            MeshMaterial2d(material.clone()),
            Transform::from_xyz(p.x * DISP_SCALE, p.y * DISP_SCALE, 0.),
            // data
            p,
        )
    }));
}

fn render_polar(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    query: Query<&Position, With<Aircraft>>,
    sensors: Query<&Position, With<Sensor>>,
) {
    const SIG_RANGE: f32 = 20.;
    const SIG_AZIMUTH: f32 = 0.2; // degrees

    let shape = meshes.add(Circle::new(2.0));
    let color = Color::srgb(1., 1., 0.);
    let material = materials.add(color);

    let mut rng = rand::rng();

    let mut points = Vec::with_capacity(query.count());
    for sen_p in sensors {
        for tar_p in query {
            let error_x = SIG_RANGE * rng.random_range(-1.0..=1.);
            let error_y = SIG_AZIMUTH * rng.random_range(-1.0..=1.);

            let dist_x = tar_p.x - sen_p.x;
            let dist_y = tar_p.y - sen_p.y;

            let pos_x = (dist_x.powi(2) + dist_y.powi(2)).sqrt();
            let pos_y = (dist_y / dist_x).atan();

            points.push(Position {
                x: (pos_x + error_x),
                y: (pos_y + error_y),
            });
        }
    }

    commands.spawn_batch(points.into_iter().map(move |p| {
        (
            PolarMeasure,
            // render
            Mesh2d(shape.clone()),
            MeshMaterial2d(material.clone()),
            Transform::from_xyz(p.x * DISP_SCALE, p.y * DISP_SCALE, 0.),
            // data
            p,
        )
    }));
}
