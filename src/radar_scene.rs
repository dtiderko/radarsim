use std::time::Duration;

use bevy::{prelude::*, time::common_conditions::on_timer};
use rand::prelude::*;

use crate::common::*;

#[derive(Component)]
struct CartesianMeasure;

pub struct RadarScene;
impl Plugin for RadarScene {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            render_cartesian.run_if(on_timer(Duration::from_secs(5 / TIME_SCALE as u64))),
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
