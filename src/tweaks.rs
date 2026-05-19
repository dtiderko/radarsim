use bevy::prelude::*;
use bevy_egui::prelude::*;

use crate::common::*;

#[derive(Resource)]
pub struct Tweaks {
    pub entity_scale: f32,
    pub time_scale: f32,

    pub cartesian_sig: f32,
    pub polar_sig_range: f32,
    pub polar_sig_azimuth: f32,
}

impl Default for Tweaks {
    fn default() -> Self {
        Self {
            entity_scale: 1.0,
            time_scale: 1.0,

            cartesian_sig: 50.0,
            polar_sig_range: 20.0,
            polar_sig_azimuth: 0.2,
        }
    }
}

pub struct TweaksUi;

impl Plugin for TweaksUi {
    fn build(&self, app: &mut App) {
        app.init_resource::<Tweaks>();

        app.add_systems(EguiPrimaryContextPass, tweaks_ui)
            .add_systems(
                Update,
                update_entity_sizes.run_if(resource_changed::<Tweaks>),
            );
    }
}

fn tweaks_ui(
    mut contexts: EguiContexts,
    mut tweaks: ResMut<Tweaks>,
    sim_time: Res<SimTime>,
    radar_sweep_counter: Res<RadarSweepCounter>,
) -> Result {
    egui::Window::new("Tweaks").show(contexts.ctx_mut()?, |ui| {
        ui.heading("Values");
        ui.label(format!("Simtime: {:.2}s", sim_time.0));
        ui.label(format!("Radar Sweeps: {}", radar_sweep_counter.0));

        ui.heading("World");
        ui.add(egui::Slider::new(&mut tweaks.entity_scale, 0.1..=4.0).text("Entity Scale"));
        ui.add(egui::Slider::new(&mut tweaks.time_scale, 0.0..=100.0).text("Time Scale"));

        ui.separator();
        ui.heading("Error");

        ui.add(egui::Slider::new(&mut tweaks.cartesian_sig, 0.0..=200.0).text("Cartesian σ (m)"));
        ui.add(
            egui::Slider::new(&mut tweaks.polar_sig_range, 0.0..=100.0).text("Polar Range σ (m)"),
        );
        ui.add(
            egui::Slider::new(&mut tweaks.polar_sig_azimuth, 0.0..=5.0)
                .text("Polar Azimuth σ (deg)"),
        );
    });

    Ok(())
}

fn update_entity_sizes(tweaks: Res<Tweaks>, entities: Query<&mut Transform, With<Mesh2d>>) {
    for mut e in entities {
        e.scale = Vec3::splat(tweaks.entity_scale);
    }
}
