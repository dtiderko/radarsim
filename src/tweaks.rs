use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use bevy_egui::prelude::*;

use crate::{chi_squared::CORRELATION_PROBS, common::*, kalman_scene::KalmanStore};

#[derive(Resource)]
pub struct Tweaks {
    /// Whether or not the camera should move and zoom automatically
    pub auto_camera: bool,
    /// How much the entities should be scaled
    /// (Useful if the screen your looking at is small)
    pub entity_scale: f32,
    /// How much the velocity arrow on the aircraft should be scaled
    /// (Useful if the screen your looking at is small)
    pub arrow_scale: f32,
    /// How fast the simulation should run
    pub time_scale: f32,

    /// The amount of sensors / radar stations to use (1 - 4)
    pub sensor_amount: usize,

    /// The error of cartesian measuements
    pub cartesian_sig: f32,
    /// The range error of polar measuements (in meters)
    pub polar_sig_range: f32,
    /// The angle error of polar measuements (in degrees)
    pub polar_sig_azimuth: f32,

    /// Whether or not to show cartesian measurements
    pub show_cartesian: bool,
    /// Whether or not to show polar measurements
    pub show_polar: bool,

    /// Maximum object speed
    /// Used in the predictions of the kalman filter
    /// in m/s
    pub kalman_vmax: f32,
    /// How much the aircraft can accelerate
    /// Used in the predictions of the kalman filter
    /// in m/s^2
    pub kalman_acc_noise: f32,
    /// Used in the expectation gate to throw away far away measuements
    /// value shown is 1 - P_c
    /// values stored is P_c
    pub kalman_correlation_prob: f32,
}

impl Default for Tweaks {
    fn default() -> Self {
        Self {
            auto_camera: true,
            entity_scale: 1.0,
            arrow_scale: 1.0,
            time_scale: 1.0,

            sensor_amount: 1,

            // according to lecture 3, slide 41
            cartesian_sig: 50.0,
            // according to lecture 3, slide 41
            polar_sig_range: 20.0,
            // according to lecture 3, slide 41
            polar_sig_azimuth: 0.2,

            show_cartesian: true,
            show_polar: true,

            kalman_vmax: 400.,
            kalman_acc_noise: 20.,
            kalman_correlation_prob: 0.005,
        }
    }
}

pub struct TweaksUi;

impl Plugin for TweaksUi {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin::default())
            .init_resource::<Tweaks>()
            .add_systems(EguiPrimaryContextPass, tweaks_ui)
            .add_systems(
                Update,
                (
                    update_entity_sizes,
                    update_cartesian_visibility,
                    update_polar_visibility,
                )
                    .run_if(resource_changed::<Tweaks>),
            );
    }
}

#[allow(clippy::too_many_arguments)]
fn tweaks_ui(
    mut contexts: EguiContexts,
    mut commands: Commands,
    mut tweaks: ResMut<Tweaks>,
    mut sim_time: ResMut<SimTime>,
    mut radar_sweep_counter: ResMut<RadarSweepCounter>,
    mut kalman_store: ResMut<KalmanStore>,
    diagnostics: Res<DiagnosticsStore>,
    cartesian_measurements: Query<Entity, With<CartesianMeasure>>,
    polar_measurements: Query<Entity, With<PolarMeasure>>,
    kalman_points: Query<Entity, With<KalmanPoint>>,
) -> Result {
    let fps = diagnostics
        .get(&bevy::diagnostic::FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|d| d.average())
        .unwrap_or(0.0);

    egui::Window::new("Tweaks").show(contexts.ctx_mut()?, |ui| {
        ui.heading("Values");
        ui.label(format!("FPS: {:.0}", fps));
        ui.label(format!("Simtime: {:.2}s", sim_time.0));
        ui.label(format!("Radar Sweeps: {}", radar_sweep_counter.0));

        ui.separator();
        ui.heading("World");
        ui.toggle_value(&mut tweaks.auto_camera, "Auto Camera");
        ui.add(egui::Slider::new(&mut tweaks.entity_scale, 0.1..=4.0).text("Entity Scale"));
        ui.add(egui::Slider::new(&mut tweaks.arrow_scale, 0.1..=16.0).text("Arrow Scale"));
        ui.add(egui::Slider::new(&mut tweaks.time_scale, 0.0..=100.0).text("Time Scale"));

        ui.separator();
        ui.heading("Amounts");
        ui.add(egui::Slider::new(&mut tweaks.sensor_amount, 1..=4).text("Sensor Amount"));

        ui.separator();
        ui.heading("Error");

        ui.add(egui::Slider::new(&mut tweaks.cartesian_sig, 0.0..=2000.0).text("Cartesian σ (m)"));
        ui.add(
            egui::Slider::new(&mut tweaks.polar_sig_range, 0.0..=2000.0).text("Polar Range σ (m)"),
        );
        ui.add(
            egui::Slider::new(&mut tweaks.polar_sig_azimuth, 0.0..=45.0)
                .text("Polar Azimuth σ (deg)"),
        );

        ui.separator();
        ui.heading("Kalman filter");
        ui.add(
            egui::Slider::new(&mut tweaks.kalman_vmax, 100.0..=600.)
                .text("Max. Object Speed (m/s)"),
        );
        ui.add(
            egui::Slider::new(&mut tweaks.kalman_acc_noise, 0.0..=100.)
                .text("Acceleration Noise (m/s^2)"),
        );
        egui::ComboBox::from_label("Correlation Probability")
            .selected_text(format!("{}", 1. - tweaks.kalman_correlation_prob))
            .show_ui(ui, |ui| {
                for p in CORRELATION_PROBS {
                    ui.selectable_value(
                        &mut tweaks.kalman_correlation_prob,
                        p,
                        format!("{}", 1. - p),
                    );
                }
            });

        ui.separator();
        ui.heading("Resets");
        if ui.button("Reset simulation").clicked() {
            sim_time.0 = 0.;
            radar_sweep_counter.0 = 0;
            *kalman_store = KalmanStore::default();
            for e in cartesian_measurements {
                commands.entity(e).despawn()
            }
            for e in polar_measurements {
                commands.entity(e).despawn()
            }
            for e in kalman_points {
                commands.entity(e).despawn()
            }
        }
        ui.toggle_value(&mut tweaks.show_cartesian, "Show cartesian measurements");
        ui.toggle_value(&mut tweaks.show_polar, "Show polar measurements");
    });

    Ok(())
}

fn update_entity_sizes(tweaks: Res<Tweaks>, entities: Query<&mut Transform, With<Mesh2d>>) {
    for mut e in entities {
        e.scale = Vec3::splat(tweaks.entity_scale);
    }
}

fn update_cartesian_visibility(
    tweaks: Res<Tweaks>,
    mut query: Query<&mut Visibility, With<CartesianMeasure>>,
) {
    for mut vis in &mut query {
        *vis = if tweaks.show_cartesian {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

fn update_polar_visibility(
    tweaks: Res<Tweaks>,
    mut query: Query<&mut Visibility, With<PolarMeasure>>,
) {
    for mut vis in &mut query {
        *vis = if tweaks.show_polar {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}
