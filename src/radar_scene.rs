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

/// ensure the radar sweep counter increases every 5s
fn update_radar_sweep_counter(
    sim_time: Res<SimTime>,
    mut radar_sweep_counter: ResMut<RadarSweepCounter>,
) {
    let cur_sweep = (sim_time.0 / RADAR_SWEEP_INTERVAL).floor() as usize;

    if cur_sweep > radar_sweep_counter.0 {
        radar_sweep_counter.0 = cur_sweep;
    }
}

/// Simulates cartesian measuements according to lecture 3, slide 41
fn render_cartesian(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    aircraft_pos: Single<&Position, With<Aircraft>>,
    tweaks: Res<Tweaks>,
    radar_sweep_counter: Res<RadarSweepCounter>,
) {
    let shape = meshes.add(Circle::new(25.0));
    let color = Color::srgb(0., 1., 1.);
    let material = materials.add(color);

    // the random number generator we have to use
    let normrnd = rand_distr::Normal::new(0., 1.).unwrap();

    // u_k
    let noise = tweaks.cartesian_sig
        * vector![
            normrnd.sample(&mut rand::rng()),
            normrnd.sample(&mut rand::rng()),
        ];

    // Normally we would calc H * vector![pos.0, vel.0, acc.0] but
    // Matrix H in the exercise is probably wrong and should be H=(I 0 0)
    // Reason: It would not make sense to add the units: m + m/s + m/s^2
    // Therefore we can omit vel and acc and with that H * ...

    commands.spawn((
        CartesianMeasure,
        // render
        Mesh2d(shape.clone()),
        MeshMaterial2d(material.clone()),
        Transform::from_xyz(aircraft_pos.x, aircraft_pos.y, -1.),
        Visibility::Visible,
        // data
        Position(aircraft_pos.0 + noise), // x_k + u_k
        TimeStep(radar_sweep_counter.0),
    ));
}

/// Simulates polar measuements according to lecture 3, slide 41
fn render_polar(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<NormalDistMaterial>>,
    aircraft_pos: Single<&Position, With<Aircraft>>,
    sensors: Query<&Position, With<Sensor>>,
    tweaks: Res<Tweaks>,
    radar_sweep_counter: Res<RadarSweepCounter>,
) {
    let color = Color::srgb(1., 1., 0.).to_linear();
    let material = materials.add(NormalDistMaterial { color });

    // the random number generator we have to use
    let normrnd = rand_distr::Normal::new(0., 1.).unwrap();

    // for each sensor we spawn the polar measuements seperately
    // fusion is done later in the kalman filter
    for sen_p in sensors {
        let error = vector![
            tweaks.polar_sig_range * normrnd.sample(&mut rand::rng()),
            tweaks.polar_sig_azimuth.to_radians() * normrnd.sample(&mut rand::rng()),
        ];

        let dist = aircraft_pos.0 - sen_p.0;
        // x -> range; y -> azimuth
        let true_polar = vector![
            (dist.x.powi(2) + dist.y.powi(2)).sqrt(),
            // in rust we have to use atan2 instead of arctan
            f32::atan2(dist.y, dist.x),
        ];
        let polar = PolarPosition(true_polar + error);

        // calc the blob to render
        let cartesian = polar.to_cartesian(sen_p.0);
        let shape = meshes.add(Ellipse::new(
            tweaks.polar_sig_range,
            polar.x * tweaks.polar_sig_azimuth.to_radians().tan(),
        ));
        let transform = Transform::from_xyz(cartesian.x, cartesian.y, -1.0)
            .with_rotation(Quat::from_rotation_z(polar.y));

        commands.spawn((
            PolarMeasure,
            // render
            MeshMaterial2d(material.clone()),
            Mesh2d(shape),
            transform,
            Visibility::Visible,
            // data
            polar,
            SensorPos(sen_p.0),
            TimeStep(radar_sweep_counter.0),
        ));
    }
}
