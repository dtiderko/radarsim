use bevy::prelude::*;
use nalgebra::{Matrix2, Matrix4, Vector2, Vector4};
use nalgebra::{matrix, vector};

use crate::chi_squared::chi_squared;
use crate::common::*;
use crate::normal_dist_material::NormalDistMaterial;
use crate::tweaks::*;

pub struct KalmanScene;
impl Plugin for KalmanScene {
    fn build(&self, app: &mut App) {
        app.init_resource::<KalmanStore>()
            .add_systems(Update, update_kalman_store)
            .add_systems(
                Update,
                render_kalman_points.run_if(resource_changed::<KalmanStore>),
            )
            .add_observer(prediction)
            .add_observer(filtering)
            .add_observer(retrodiction);
    }
}

/// This event gets triggered by the update_kalman_store function once the kalman
/// filter is initiated and a new measurement is available.
///
/// Listening is the prediction function.
#[derive(Event)]
struct NewMeasurementEvent(pub TimeStep);

/// This event gets triggered by the prediction function once the prediction got
/// made.
///
/// Listening is the filtering function.
#[derive(Event)]
struct KalmanPredEvent(pub TimeStep);

/// This event gets triggered by the filtering function once the filtering and
/// expectation gating got done.
///
/// Listening is the retrodiction function.
#[derive(Event)]
struct KalmanFilterEvent(pub TimeStep);

/// This compontent is used by the render_kalman_points function to efficiently
/// decide which points to keep and which to destroy in favor of newer points.
#[derive(Component)]
struct BasedOn(pub TimeStep);

/// Represents a point guessed by the kalman filter
/// It can be interpreted as a normal distribution
#[derive(Clone, Debug)]
pub struct Gaussian {
    /// inferred point with (pos.x, pos.y, vel.x, vel.y)^T
    mean: Vector4<f32>,
    /// error of the inferred point
    covariance: Matrix4<f32>,
}

/// Stores the results of the kalman filter in a upper right corner matrix
#[derive(Resource)]
pub struct KalmanStore {
    /// actual measurements in Cartesian coordinates
    zs: Vec<Vector2<f32>>,
    /// error covariance matricies
    rs: Vec<Matrix2<f32>>,

    /// Matrix with:
    /// - i as rows (the timestep this data was meant for)
    /// - given as colmuns (timestep of the sensor data it is based on)
    ///
    /// Accessed like: data[i][given]
    data: Vec<Vec<Option<Gaussian>>>,
}
impl Default for KalmanStore {
    fn default() -> Self {
        Self {
            zs: vec![],
            rs: vec![],
            data: vec![vec![]],
        }
    }
}
impl KalmanStore {
    /// Get the Gaussian g_{i|given} as seen in the book and lectures
    pub fn get(&self, i: usize, given: usize) -> Option<&Gaussian> {
        let givens = self.data.get(i);
        let ret = givens.and_then(|x| x.get(given));
        ret.and_then(|x| x.into())
    }

    /// Set the Gaussian at g_{i|given} as seen in the book and lectures
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

/// Calculate the rendered ellipse (rotation and size) of a given gaussian.
fn calc_ellipse(gaussian: &Gaussian) -> (Ellipse, Transform) {
    let eig = gaussian
        .covariance
        .fixed_view::<2, 2>(0, 0)
        .symmetric_eigen();

    // calc error covariance half-width and half-height
    let half_width = eig.eigenvalues[0].sqrt();
    let half_height = eig.eigenvalues[1].sqrt();
    let shape = Ellipse::new(half_width, half_height);

    // calc rotation of our first guess
    let lambda1 = eig.eigenvectors.column(0);
    let theta = f32::atan2(lambda1.y, lambda1.x);
    let rotation = Quat::from_rotation_z(theta);
    // and combine it with the position of the guessed point
    let transform =
        Transform::from_xyz(gaussian.mean.x, gaussian.mean.y, 0.).with_rotation(rotation);

    (shape, transform)
}

/// Render all the latest kalman points (x_{latest_k|lastest_k})
fn render_kalman_points(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<NormalDistMaterial>>,
    kalman_store: Res<KalmanStore>,
    kalman_points: Query<(Entity, &TimeStep, &BasedOn), With<KalmanPoint>>,
) {
    // points should be pink gaussians
    let color = Color::srgb(1., 0., 0.5).to_linear();
    let material = MeshMaterial2d(materials.add(NormalDistMaterial { color }));

    let mut entities = Vec::with_capacity(kalman_store.data.len());
    for (k, row) in kalman_store.data.iter().enumerate() {
        // find the latest value at timestep k
        let mut point = None;
        let mut target_base = None;
        for (b, x) in row.iter().enumerate().rev() {
            if x.is_some() {
                point = x.as_ref();
                target_base = Some(b);
                break;
            }
        }
        if point.is_none() {
            continue;
        }
        let point = point.unwrap();

        // despawn the current entity for this timestep
        let mut entity = None;
        let mut cur_base = None;
        for (e, ek, b) in kalman_points {
            if ek.0 == k {
                entity = Some(e);
                cur_base = Some(b.0.0);
                break;
            }
        }

        // target entity already rendered
        // -> skip it
        if target_base == cur_base {
            continue;
        }

        // despawn old entity
        if let Some(entity) = entity {
            commands.entity(entity).despawn();
        }

        // create target entity
        let (shape, transform) = calc_ellipse(point);
        entities.push((
            KalmanPoint,
            Mesh2d(meshes.add(shape)),
            material.clone(),
            transform,
            TimeStep(k),
            BasedOn(TimeStep(target_base.unwrap())),
        ));
    }
    commands.spawn_batch(entities);
}

/// Update the kalman store with the newest measurements and start the initiation
/// or the prediction
///
/// This will also fusion all measurements of timestep k, calc the cartesian
/// position of the fusioned result and calc the error (R) of the fusioned result.
fn update_kalman_store(
    mut commands: Commands,
    mut kalman_store: ResMut<KalmanStore>,
    tweaks: Res<Tweaks>,
    measurements: Query<(&PolarPosition, &SensorPos, &TimeStep), With<PolarMeasure>>,
) {
    let latest_k = kalman_store.zs.len();
    let latest_measurements = measurements
        .iter()
        .sort_by_key::<&TimeStep, _>(|x| x.0)
        .filter(|x| x.2.0 == latest_k);

    let with_inv_rs: Vec<(Vector2<f32>, Matrix2<f32>)> = latest_measurements
        .map(|(z, sen_p, _)| {
            (
                z.to_cartesian(sen_p.0).0,
                calc_r_k(
                    z.y,
                    z.x,
                    tweaks.polar_sig_azimuth.to_radians(),
                    tweaks.polar_sig_range,
                )
                .try_inverse()
                .unwrap(),
            )
        })
        .collect();

    // if there is a new measurement
    if !with_inv_rs.is_empty() {
        // fusion of measurements
        // with only one sensor we have let r = r; since R^(-1)^(-1) = R
        let r = with_inv_rs
            .iter()
            .map(|(_, r)| r)
            .sum::<Matrix2<f32>>()
            .try_inverse()
            .unwrap();
        // with only one sensor we have let z = z; since R * R^-1 = I
        let z = r * with_inv_rs.iter().map(|(z, r)| r * z).sum::<Vector2<f32>>();

        // add measurement to store
        kalman_store.zs.push(z);
        kalman_store.rs.push(r);

        // if we need to initiate
        if kalman_store.get(0, 0).is_none() {
            initiate(&tweaks, &mut kalman_store);
        } else {
            commands.trigger(NewMeasurementEvent(TimeStep(latest_k)));
        }
    }
}

/// Initiate the kalman filter according to book p. 60, section 3.1.4
fn initiate(tweaks: &Tweaks, kalman_store: &mut KalmanStore) {
    // our first measurement
    let z0 = &kalman_store.zs[0];
    // error covariance matrix
    let r0 = &kalman_store.rs[0];

    // our first predicted state x_{0|0}
    let x00 = vector![
        z0.x, // pos x
        z0.y, // pos y
        0.,   // vel x
        0.,   // vel y
    ];

    // our first covariance matrix
    let p00 = matrix![
        r0[(0,0)], r0[(0,1)], 0., 0.;
        r0[(1,0)], r0[(1,1)], 0., 0.;
        0., 0., tweaks.kalman_vmax.powi(2), 0.;
        0., 0., 0., tweaks.kalman_vmax.powi(2);
    ];

    kalman_store.set(
        0,
        0,
        Gaussian {
            mean: x00,
            covariance: p00,
        },
    );
}

/// Make a kalman prediction with the piecewise constant white acceleration model
/// according to lecture 4, slide 18f
fn prediction(
    event: On<NewMeasurementEvent>,
    tweaks: Res<Tweaks>,
    mut commands: Commands,
    mut kalman_store: ResMut<KalmanStore>,
) {
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

/// Ignore all measurements outside of the expectiation gate according to book
/// p. 61f, section 3.2.2 and filter the prediction with actual measurements
/// according to book p.65, section 3.3.1
fn filtering(
    event: On<KalmanPredEvent>,
    mut commands: Commands,
    mut kalman_store: ResMut<KalmanStore>,
    tweaks: Res<Tweaks>,
) {
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
    let z_k = &kalman_store.zs[*k];
    // measurement error matrix
    let r_k = &kalman_store.rs[*k];

    let h = matrix![
        1., 0., 0., 0.;
        0., 1., 0., 0.;
    ];

    // innovation covariance
    let s_k_kp = h * p_k_kp * h.transpose() + r_k;
    let s_k_kp_inv = s_k_kp.try_inverse().unwrap();
    // kalman gain matrix
    let w_k_kp = p_k_kp * h.transpose() * s_k_kp_inv;

    // innovation
    let v = z_k - h * x_k_kp;

    let innovation_square = v.transpose() * s_k_kp_inv * v;
    let max_distance = chi_squared(tweaks.kalman_correlation_prob);

    let x_k_k;
    let p_k_k;

    // expectation gate
    if innovation_square[(0, 0)] > max_distance {
        // skip this measurement by accepting the prediction
        x_k_k = x_k_kp;
        p_k_k = p_k_kp;
    } else {
        x_k_k = x_k_kp + w_k_kp * v;
        p_k_k = p_k_kp - w_k_kp * s_k_kp * w_k_kp.transpose();
    }

    // update inferred data in store
    kalman_store.set(
        *k,
        *k,
        Gaussian {
            mean: x_k_k,
            covariance: p_k_k,
        },
    );

    // trigger retrodiction
    commands.trigger(KalmanFilterEvent(k));
}

/// Perform the retrodiction using the fixed interval smoothing according to book
/// p. 72, section 3.4.1
fn retrodiction(
    event: On<KalmanFilterEvent>,
    mut kalman_store: ResMut<KalmanStore>,
    kalman_points: Query<&TimeStep, With<KalmanPoint>>,
) {
    let k = event.0;

    // sort in descending order (newest to oldest)
    let sorted = kalman_points
        .iter()
        .sort_by_key::<&TimeStep, _>(|x| x.0)
        .rev();

    for l in sorted {
        let l = l.0;

        // current point
        let Gaussian {
            mean: x_l_l,
            covariance: p_l_l,
        } = kalman_store.get(l, l).unwrap();
        let Gaussian {
            mean: x_ln_l,
            covariance: p_ln_l,
        } = kalman_store.get(l + 1, l).unwrap();
        let Gaussian {
            mean: x_ln_k,
            covariance: p_ln_k,
        } = kalman_store.get(l + 1, *k).unwrap();

        // transition matrix
        // according to lecture 4, slide 19
        let f_ln_l = matrix![
            1., 0., RADAR_SWEEP_INTERVAL, 0.;
            0., 1., 0., RADAR_SWEEP_INTERVAL;
            0., 0., 1., 0.;
            0., 0., 0., 1.;
        ];
        let w_l_ln = p_l_l * f_ln_l.transpose() * p_ln_l.try_inverse().unwrap();

        let x_l_k = x_l_l + w_l_ln * (x_ln_k - x_ln_l);
        let p_l_k = p_l_l + w_l_ln * (p_ln_k - p_ln_l) * w_l_ln.transpose();

        kalman_store.set(
            l,
            *k,
            Gaussian {
                mean: x_l_k,
                covariance: p_l_k,
            },
        );
    }
}
