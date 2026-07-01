use bevy::prelude::*;
use nalgebra::{Vector2, vector};

pub const RADAR_SWEEP_INTERVAL: f32 = 5.0;

#[derive(Resource, Deref, DerefMut)]
pub struct SimTime(pub f32);

#[derive(Resource, Component, Clone, Copy, Deref, DerefMut)]
pub struct RadarSweepCounter(pub usize);

#[derive(Component)]
pub struct CartesianMeasure;

#[derive(Component)]
pub struct PolarMeasure;

#[derive(Component)]
pub struct KalmanPoint;

#[derive(Component, Debug)]
pub struct Aircraft;

#[derive(Component, Debug)]
pub struct Sensor;

#[derive(Clone, Component, Copy, Debug, Deref, DerefMut, Eq, Ord, PartialEq, PartialOrd)]
pub struct TimeStep(pub usize);

#[derive(Component, Debug, Deref, DerefMut, Clone)]
pub struct SensorPos(pub Vector2<f32>);

/// x -> range
/// y -> azimuth in radians
#[derive(Component, Debug, Default, Deref, DerefMut, Clone)]
pub struct PolarPosition(pub Vector2<f32>);
impl PolarPosition {
    pub fn to_cartesian(&self, sensor_pos: Vector2<f32>) -> Position {
        let range = self.x;
        let azimuth = self.y;
        Position(range * vector![azimuth.cos(), azimuth.sin()] + sensor_pos)
    }
}

#[derive(Component, Debug, Default, Deref, DerefMut, Clone)]
pub struct Position(pub Vector2<f32>);
