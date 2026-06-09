use bevy::prelude::*;
use nalgebra::vector;
use rand::prelude::*;

use crate::common::*;
use crate::normal_dist_material::NormalDistMaterial;
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
    let cur_sweep = (sim_time.0 / 5.).floor() as usize;

    if cur_sweep > radar_sweep_counter.0 {
        radar_sweep_counter.0 = cur_sweep;
    }
}

fn render_cartesian(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    query: Query<&Position, With<Aircraft>>,
    tweaks: Res<Tweaks>,
    radar_sweep_counter: Res<RadarSweepCounter>,
) {
    let shape = meshes.add(Circle::new(25.0));
    let color = Color::srgb(0., 1., 1.);
    let material = materials.add(color);

    let normrnd = rand_distr::Normal::new(0., 1.).unwrap();

    let mut points = Vec::with_capacity(query.count());
    for p in query {
        let noise = tweaks.cartesian_sig
            * vector![
                normrnd.sample(&mut rand::rng()),
                normrnd.sample(&mut rand::rng()),
            ];

        // Normally we would calc H * vector![pos.0, vel.0, acc.0] but
        // Matrix H in the exercise is probably wrong and should be H=(I 0 0)
        // Reason: It would not make sense to add the units: m + m/s + m/s^2
        // Therefore we can omit vel and acc and with that H * ...

        points.push(Position(p.0 + noise));
    }

    let k = RadarSweepCounter(radar_sweep_counter.0);
    commands.spawn_batch(points.into_iter().map(move |p| {
        (
            CartesianMeasure,
            // render
            Mesh2d(shape.clone()),
            MeshMaterial2d(material.clone()),
            Transform::from_xyz(p.x, p.y, -1.),
            Visibility::Visible,
            // data
            p,
            k,
        )
    }));
}

fn render_polar(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<NormalDistMaterial>>,
    query: Query<&Position, With<Aircraft>>,
    sensors: Query<&Position, With<Sensor>>,
    tweaks: Res<Tweaks>,
    radar_sweep_counter: Res<RadarSweepCounter>,
) {
    let color = Color::srgb(1., 1., 0.).to_linear();
    let material = materials.add(NormalDistMaterial { color });

    let normrnd = rand_distr::Normal::new(0., 1.).unwrap();

    // TODO proper sensor data fusion (see email)
    let mut points = Vec::with_capacity(query.count());
    for sen_p in sensors {
        for tar_p in query {
            let error = vector![
                tweaks.polar_sig_range * normrnd.sample(&mut rand::rng()),
                tweaks.polar_sig_azimuth.to_radians() * normrnd.sample(&mut rand::rng()),
            ];

            let dist = tar_p.0 - sen_p.0;
            let true_polar = vector![
                (dist.x.powi(2) + dist.y.powi(2)).sqrt(), // range
                dist.y.atan2(dist.x),                     // azimuth // arctan(dist_y / dist_x)
            ];

            let polar = true_polar + error;
            let cartesian = polar.x * vector![polar.y.cos(), polar.y.sin()] + sen_p.0;

            let shape = meshes.add(Ellipse::new(
                tweaks.polar_sig_range,
                polar.x * tweaks.polar_sig_azimuth.to_radians().tan(),
            ));

            let transform = Transform::from_xyz(cartesian.x, cartesian.y, -1.0)
                .with_rotation(Quat::from_rotation_z(polar.y));

            points.push((
                transform,
                Mesh2d(shape),
                PolarPosition(polar),
                SensorPos(sen_p.0),
            ));
        }
    }

    let k = RadarSweepCounter(radar_sweep_counter.0);
    commands.spawn_batch(
        points
            .into_iter()
            .map(move |(transform, mesh, pos, sen_p)| {
                (
                    PolarMeasure,
                    // render
                    MeshMaterial2d(material.clone()),
                    mesh,
                    transform,
                    Visibility::Visible,
                    // data
                    pos,
                    sen_p,
                    k,
                )
            }),
    );
}
