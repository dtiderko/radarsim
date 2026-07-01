use bevy::prelude::*;
use nalgebra::vector;

use crate::{common::*, tweaks::Tweaks};

pub struct RealScene;
impl Plugin for RealScene {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_aircraft)
            .add_systems(Update, setup_sensors.run_if(resource_changed::<Tweaks>))
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

fn setup_sensors(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    existing_sensors: Query<Entity, With<Sensor>>,
    tweaks: Res<Tweaks>,
) {
    let shape = meshes.add(Circle::new(100.0));
    let color = Color::srgb(0., 1., 0.);

    let poss = vec![
        vector![5000., 5000.],
        vector![-20000., 10000.],
        vector![10000., 12000.],
        vector![-7000., -15000.],
    ];

    // first kill all remaining sensors
    existing_sensors
        .iter()
        .for_each(|e| commands.entity(e).despawn());

    // then spawn the new amount in
    let sensors: Vec<_> = poss
        .into_iter()
        .take(tweaks.sensor_amount)
        .map(|pos| {
            (
                Sensor,
                // render
                Mesh2d(shape.clone()),
                MeshMaterial2d(materials.add(color)),
                Transform::from_xyz(pos.x, pos.y, 0.),
                // data
                Position(pos),
            )
        })
        .collect();
    commands.spawn_batch(sensors);
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
    aircraft: Single<(&Position, &Velocity, &Acceleration), With<Aircraft>>,
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
    gizmos.arrow_2d(
        Vec2::new(aircraft.0.x, aircraft.0.y),
        Vec2::new(
            aircraft.0.x + aircraft.2.x * tweaks.arrow_scale,
            aircraft.0.y + aircraft.2.y * tweaks.arrow_scale,
        ),
        Color::srgb(1., 1., 0.),
    );
}
