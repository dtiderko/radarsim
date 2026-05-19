use bevy::prelude::*;
use rand::prelude::*;

use crate::common::*;
use crate::tweaks::*;

pub struct RadarScene;
impl Plugin for RadarScene {
    fn build(&self, app: &mut App) {
        app.insert_resource(RadarSweepCounter(0))
            .add_systems(Update, update_radar_sweep_counter)
            .add_systems(
                Update,
                (render_cartesian, render_polar).run_if(resource_changed::<RadarSweepCounter>),
            );
    }
}

fn update_radar_sweep_counter(
    sim_time: Res<SimTime>,
    mut radar_sweep_counter: ResMut<RadarSweepCounter>,
) {
    let cur_sweep = (sim_time.0 / 5.).floor() as u64;

    if cur_sweep > radar_sweep_counter.0 {
        radar_sweep_counter.0 = cur_sweep;
    }
}

fn render_cartesian(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    query: Query<(&Position, &Velocity, &Acceleration), With<Aircraft>>,
    tweaks: Res<Tweaks>,
) {
    let shape = meshes.add(Circle::new(2.0));
    let color = Color::srgb(0., 1., 1.);
    let material = materials.add(color);

    let mut rng = rand::rng();

    let mut points = Vec::with_capacity(query.count());
    for (p, v, a) in query {
        points.push(Position {
            x: (p.x + v.x + a.x + tweaks.cartesian_sig * rng.random_range(-1.0..=1.)),
            y: (p.y + v.y + a.y + tweaks.cartesian_sig * rng.random_range(-1.0..=1.)),
        });
    }

    commands.spawn_batch(points.into_iter().map(move |p| {
        (
            CartesianMeasure,
            // render
            Mesh2d(shape.clone()),
            MeshMaterial2d(material.clone()),
            Transform::from_xyz(p.x, p.y, 0.),
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
    tweaks: Res<Tweaks>,
) {
    let shape = meshes.add(Circle::new(2.0));
    let color = Color::srgb(1., 1., 0.);
    let material = materials.add(color);

    let mut rng = rand::rng();

    let mut points = Vec::with_capacity(query.count());
    for sen_p in sensors {
        for tar_p in query {
            let error_x = tweaks.polar_sig_range * rng.random_range(-1.0..=1.);
            let error_y = tweaks.polar_sig_range * rng.random_range(-1.0..=1.);

            let dist_x = tar_p.x - sen_p.x;
            let dist_y = tar_p.y - sen_p.y;

            let pos_x = (dist_x.powi(2) + dist_y.powi(2)).sqrt();

            let pos_y = dist_y.atan2(dist_x); // arctan(dist_y / dist_x)

            let range = pos_x + error_x;
            let azimuth = pos_y + error_y;

            let cartesian_x = range * azimuth.cos() + sen_p.x;
            let cartesian_y = range * azimuth.sin() + sen_p.y;

            points.push((
                PolarPosition { range, azimuth },
                Transform::from_xyz(cartesian_x, cartesian_y, 0.0),
            ));
        }
    }

    commands.spawn_batch(points.into_iter().map(move |(pos, transform)| {
        (
            PolarMeasure,
            // render
            Mesh2d(shape.clone()),
            MeshMaterial2d(material.clone()),
            transform,
            // data
            pos,
        )
    }));
}
