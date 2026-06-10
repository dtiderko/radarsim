use bevy::prelude::*;
use nalgebra::{Matrix2, Matrix4, Vector4};
use nalgebra::{matrix, vector};

use crate::common::*;
use crate::normal_dist_material::NormalDistMaterial;
use crate::tweaks::*;

struct InferredPoint {
    /// inferred point with (pos.x, pos.y, vel.x, vel.y)^T
    mean: Vector4<f32>,
    /// error of the inferred point
    covariance: Matrix4<f32>,
}

#[derive(Resource, Default)]
pub struct KalmanStore {
    /// actual measurements in Cartesian coordinates
    zs: Vec<(PolarPosition, SensorPos)>,
    /// inferred positions of entities
    inferred: Vec<InferredPoint>,
}

pub struct KalmanScene;
impl Plugin for KalmanScene {
    fn build(&self, app: &mut App) {
        app.init_resource::<KalmanStore>()
            .add_systems(Update, update_kalman_store);
    }
}

/// calculate the error covariance matrix R_{k} (cartesian ) given a polar measurement
/// Based on Book p.37, section 2.3.1
///
/// err_phi is expected to be given in radians
fn r_k(phi_k: f32, r_k: f32, err_phi: f32, err_r: f32) -> Matrix2<f32> {
    // rotation matrix
    let d_pk = matrix![
        phi_k.cos(), -phi_k.sin();
        phi_k.sin(), phi_k.cos();
    ];

    let r_polar = matrix![
        err_r.powi(2), 0.;
        0., r_k.powi(2) * err_phi.powi(2);
    ];

    d_pk * r_polar * d_pk.transpose()
}

fn update_kalman_store(
    tweaks: Res<Tweaks>,
    mut kalman_store: ResMut<KalmanStore>,
    measurements: Query<(&PolarPosition, &SensorPos, &RadarSweepCounter), With<PolarMeasure>>,

    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<NormalDistMaterial>>,
) {
    let latest_k = kalman_store.zs.len();
    let latest_measurements: Vec<(&PolarPosition, &SensorPos, &RadarSweepCounter)> = measurements
        .iter()
        .sort_by_key::<&RadarSweepCounter, _>(|x| x.0)
        .filter(|x| x.2.0 == latest_k)
        .collect();
    assert!(
        latest_measurements.len() <= 1,
        "expected multiple measurements per radar sweep to have been fusioned"
    );

    // if there is a new measurement
    if let Some((pos, sen_p, k)) = latest_measurements.first() {
        assert!(k.0 == kalman_store.zs.len()); // truely sorted
        kalman_store.zs.push(((*pos).clone(), (*sen_p).clone()));
    }

    // if we need to initiate
    // runs only for the first point
    if kalman_store.inferred.is_empty() && kalman_store.zs.len() == 1 {
        initiate(
            &tweaks,
            &mut commands,
            &mut kalman_store,
            &mut meshes,
            &mut materials,
        );
    }
}

fn initiate(
    tweaks: &Tweaks,
    commands: &mut Commands,
    kalman_store: &mut KalmanStore,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<NormalDistMaterial>,
) {
    // our first measurement
    let (z0_polar, sen_p) = &kalman_store.zs[0];
    let z0 = z0_polar.to_cartesian(sen_p.0);

    // our first predicted state x_{0|0}
    let x00 = vector![
        z0.x, // pos x
        z0.y, // pos y
        0.,   // vel x
        0.,   // vel y
    ];

    // error covariance matrix R_{0}
    let r0 = r_k(
        z0_polar.y,
        z0_polar.x,
        tweaks.polar_sig_azimuth.to_radians(),
        tweaks.polar_sig_range,
    );
    // our first covariance matrix
    let p00 = matrix![
        r0[(0,0)], r0[(0,1)], 0., 0.;
        r0[(1,0)], r0[(1,1)], 0., 0.;
        0., 0., tweaks.kalman_vmax.powi(2), 0.;
        0., 0., 0., tweaks.kalman_vmax.powi(2);
    ];

    // to render the ellipse
    let (material, mesh, transform) = {
        let eig = r0.symmetric_eigen();

        // calc error covariance half-width and half-height
        let half_width = eig.eigenvalues[0].sqrt();
        let half_height = eig.eigenvalues[1].sqrt();
        let shape = Ellipse::new(half_width, half_height);

        // points should be pink gaussians
        let color = Color::srgb(1., 0., 0.5).to_linear();
        let material = NormalDistMaterial { color };

        // calc rotation of our first guess
        let lambda1 = eig.eigenvectors.column(0);
        let theta = f32::atan2(lambda1.y, lambda1.x);
        let rotation = Quat::from_rotation_z(theta);
        // and combine it with the position of the guessed point
        let transform = Transform::from_xyz(x00.x, x00.y, 0.).with_rotation(rotation);

        (
            MeshMaterial2d(materials.add(material)),
            Mesh2d(meshes.add(shape)),
            transform,
        )
    };
    commands.spawn((KalmanPoint, material, mesh, transform));

    kalman_store.inferred.push(InferredPoint {
        mean: x00,
        covariance: p00,
    });
}
