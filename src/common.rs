use bevy::prelude::*;
use nalgebra::{Vector2, vector};

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

#[derive(Component, Debug, Default, Deref, DerefMut, Clone)]
pub struct Velocity(pub Vector2<f32>);

#[derive(Component, Debug, Default, Deref, DerefMut, Clone)]
pub struct Acceleration(pub Vector2<f32>);
