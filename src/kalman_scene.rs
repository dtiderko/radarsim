use bevy::prelude::*;
use nalgebra::{Matrix2, Matrix4, Vector4};
use nalgebra::{matrix, vector};

use crate::common::*;
use crate::normal_dist_material::NormalDistMaterial;
use crate::tweaks::*;

#[derive(Event)]
struct NewMeasurementEvent(pub TimeStep);

#[derive(Event)]
struct KalmanPredEvent(pub TimeStep);

#[derive(Clone, Debug)]
pub struct Gaussian {
    /// inferred point with (pos.x, pos.y, vel.x, vel.y)^T
    mean: Vector4<f32>,
    /// error of the inferred point
    covariance: Matrix4<f32>,
}

#[derive(Resource)]
pub struct KalmanStore {
    /// actual measurements in Cartesian coordinates
    zs: Vec<(PolarPosition, SensorPos)>,

    /// Matrix with:
    /// - i as rows (the timestep this data was meant for)
    /// - given as colmuns (timestep of the sensor data it is based on)
    data: Vec<Vec<Option<Gaussian>>>,
}
impl Default for KalmanStore {
    fn default() -> Self {
        Self {
            zs: vec![],
            data: vec![vec![]],
        }
    }
}
impl KalmanStore {
    /// Get the Gaussian g_{i|given} as seen in the book
    pub fn get(&self, i: usize, given: usize) -> Option<&Gaussian> {
        let givens = self.data.get(i);
        let ret = givens.and_then(|x| x.get(given));
        ret.and_then(|x| x.into())
    }

    /// Set the Gaussian at g_{i|given} as seen in the book
    pub fn set(&mut self, i: usize, given: usize, gaussian: Gaussian) {
        // fill in missing rows
        let num_cols = self.data.first().map(|x| x.len()).unwrap_or(0);
        if i >= self.data.len() {
            let missing_rows = 1 + i - self.data.len();
            let filler_row: Vec<Option<Gaussian>> = (0..num_cols).map(|_| None).collect();

            let rows = (0..missing_rows).map(|_| filler_row.clone());
            self.data.extend(rows);
        }

        // fill in missing cols
        if given >= num_cols {
            let missing_cols = 1 + given - num_cols;
            for r in &mut self.data {
                let cols = (0..missing_cols).map(|_| None);
                r.extend(cols);
            }
        }

        // set the data
        self.data[i][given] = Some(gaussian);
    }
}

pub struct KalmanScene;
impl Plugin for KalmanScene {
    fn build(&self, app: &mut App) {
        app.init_resource::<KalmanStore>()
            .add_systems(Update, update_kalman_store)
            .add_observer(prediction)
            .add_observer(filtering);
    }
}

/// calculate the error covariance matrix R_{k} (cartesian ) given a polar measurement
/// Based on Book p.37, section 2.3.1
///
/// err_phi is expected to be given in radians
fn calc_r_k(phi_k: f32, r_k: f32, err_phi: f32, err_r: f32) -> Matrix2<f32> {
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

fn render_gaussian(
    commands: &mut Commands,
    materials: &mut Assets<NormalDistMaterial>,
    meshes: &mut Assets<Mesh>,
    entity_type: impl Component,
    mean: Vector4<f32>,
    covariance: Matrix2<f32>,
    id: TimeStep,
) {
    let eig = covariance.symmetric_eigen();

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
    let transform = Transform::from_xyz(mean.x, mean.y, 0.).with_rotation(rotation);

    commands.spawn((
        entity_type,
        MeshMaterial2d(materials.add(material)),
        Mesh2d(meshes.add(shape)),
        transform,
        id,
    ));
}

fn update_kalman_store(
    tweaks: Res<Tweaks>,
    mut kalman_store: ResMut<KalmanStore>,
    measurements: Query<(&PolarPosition, &SensorPos, &TimeStep), With<PolarMeasure>>,

    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<NormalDistMaterial>>,
) {
    let latest_k = kalman_store.zs.len();
    let latest_measurements: Vec<(&PolarPosition, &SensorPos, &TimeStep)> = measurements
        .iter()
        .sort_by_key::<&TimeStep, _>(|x| x.0)
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

        // if we need to initiate
        if kalman_store.get(0, 0).is_none() {
            initiate(
                &tweaks,
                &mut commands,
                &mut kalman_store,
                &mut meshes,
                &mut materials,
            );
        } else {
            commands.trigger(NewMeasurementEvent(TimeStep(k.0)));
        }
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
    let r0 = calc_r_k(
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

    render_gaussian(
        commands,
        materials,
        meshes,
        KalmanPoint,
        x00,
        r0,
        TimeStep(0),
    );

    kalman_store.set(
        0,
        0,
        Gaussian {
            mean: x00,
            covariance: p00,
        },
    );
}

fn prediction(
    event: On<NewMeasurementEvent>,
    tweaks: Res<Tweaks>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<NormalDistMaterial>>,
    mut kalman_store: ResMut<KalmanStore>,
) {
    println!("kalman pred");

    let k = event.0;

    // get our last guess
    let (x_kp_kp, p_kp_kp) = match kalman_store.get(*k - 1, *k - 1) {
        Some(x) => (x.mean, x.covariance),
        None => {
            warn!("kalman predictions are not running in the correct order!");
            return;
        }
    };

    // transition matrix
    // according to vl 4, p.19
    let f_k_kp = matrix![
        1., 0., RADAR_SWEEP_INTERVAL, 0.;
        0., 1., 0., RADAR_SWEEP_INTERVAL;
        0., 0., 1., 0.;
        0., 0., 0., 1.;
    ];

    // process noise covariance
    // according to vl 4, p.19
    let d_k_kp = tweaks.kalman_acc_noise.powi(2)
        * matrix![
            0.25 * RADAR_SWEEP_INTERVAL.powi(4), 0., 0.5 * RADAR_SWEEP_INTERVAL.powi(3), 0.;
            0., 0.25 * RADAR_SWEEP_INTERVAL.powi(4), 0., 0.5 * RADAR_SWEEP_INTERVAL.powi(3);
            0.5 * RADAR_SWEEP_INTERVAL.powi(3), 0., RADAR_SWEEP_INTERVAL.powi(2), 0.;
            0., 0.5 * RADAR_SWEEP_INTERVAL.powi(3), 0., RADAR_SWEEP_INTERVAL.powi(2);
        ];

    // calc current guess
    // according to vl 4, p. 18 and book p. 61, section 3.2.1
    let x_k_kp = f_k_kp * x_kp_kp;
    let p_k_kp = f_k_kp * p_kp_kp * f_k_kp.transpose() + d_k_kp;

    render_gaussian(
        &mut commands,
        &mut materials,
        &mut meshes,
        KalmanPoint,
        x_k_kp,
        p_k_kp.fixed_view::<2, 2>(0, 0).into(),
        event.0,
    );

    // add our new guess as point
    kalman_store.set(
        *k,
        *k - 1,
        Gaussian {
            mean: x_k_kp,
            covariance: p_k_kp,
        },
    );

    // notify our filtering step about this new point
    commands.trigger(KalmanPredEvent(k));
}

fn filtering(
    event: On<KalmanPredEvent>,
    tweaks: Res<Tweaks>,
    mut commands: Commands,
    mut kalman_store: ResMut<KalmanStore>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<NormalDistMaterial>>,
    kalman_points: Query<(Entity, &TimeStep), With<KalmanPoint>>,
) {
    println!("kalman filter");

    let k = event.0;

    // predicted point
    let (x_k_kp, p_k_kp) = match kalman_store.get(*k, *k - 1) {
        Some(x) => (x.mean, x.covariance),
        None => {
            warn!("kalman filterings are not running in the correct order!");
            return;
        }
    };
    // sensor measurement
    let z = &kalman_store.zs[*k];
    let z_k = z.0.to_cartesian(z.1.0);

    // measurement error matrix
    let r_k = calc_r_k(
        z.0.y,
        z.0.x,
        tweaks.polar_sig_azimuth.to_radians(),
        tweaks.polar_sig_range,
    );

    let h = matrix![
        1., 0., 0., 0.;
        0., 1., 0., 0.;
    ];

    // innovation covariance
    let s_k_kp = h * p_k_kp * h.transpose() + r_k;
    // kalman gain matrix
    let w_k_kp = p_k_kp * h.transpose() * s_k_kp.try_inverse().unwrap();

    // innovation
    let v = z_k.0 - h * x_k_kp;

    let x_k_k = x_k_kp + w_k_kp * v;
    let p_k_k = p_k_kp - w_k_kp * s_k_kp * w_k_kp.transpose();

    // update inferred data in store
    kalman_store.set(
        *k,
        *k,
        Gaussian {
            mean: x_k_k,
            covariance: p_k_k,
        },
    );

    // update visualisation
    let last_points: Vec<Entity> = kalman_points
        .iter()
        .filter(|(_, id)| **id == k)
        .map(|(e, _)| e)
        .collect();
    assert!(
        last_points.len() == 1,
        "expected one kalman point per computed time step"
    );
    commands.entity(last_points[0]).despawn();
    render_gaussian(
        &mut commands,
        &mut materials,
        &mut meshes,
        KalmanPoint,
        x_k_k,
        p_k_k.fixed_view::<2, 2>(0, 0).into(),
        event.0,
    );
}
