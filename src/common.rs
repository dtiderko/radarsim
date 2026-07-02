use bevy::prelude::*;
use nalgebra::{Vector2, vector};

/// The amount of seconds between each radar sweep.
///
/// Also used as delta time for some calculations in the kalman filter
///
/// According to lecture 3, slide 39
pub const RADAR_SWEEP_INTERVAL: f32 = 5.0;

/// Seconds passed in the simulation
///
/// Might be streched by tweaks.time_scale
#[derive(Resource, Deref, DerefMut)]
pub struct SimTime(pub f32);

/// Amount of sweeps the radar has already done
#[derive(Resource, Component, Clone, Copy, Deref, DerefMut)]
pub struct RadarSweepCounter(pub usize);

/// Marks the entity of a measurement in the cartesian space
#[derive(Component)]
pub struct CartesianMeasure;

/// Marks the entity of a measurement in the polar space
#[derive(Component)]
pub struct PolarMeasure;

/// Marks the entity of a guess by the kalman filter
#[derive(Component)]
pub struct KalmanPoint;

/// Marks the entity of the aircraft
#[derive(Component, Debug)]
pub struct Aircraft;

/// Marks the entity of a sensor / radar
#[derive(Component, Debug)]
pub struct Sensor;

/// The timestep or k which this measurement or guess belongs to
///
/// It basically stores at which value the RadarSweepCounter was when this
/// measurement, or the guess based on that, was made
#[derive(Clone, Component, Copy, Debug, Deref, DerefMut, Eq, Ord, PartialEq, PartialOrd)]
pub struct TimeStep(pub usize);

/// The position of a sensor. It is given in the Cartesian / kalman / absoulte
/// space
///
/// PolarMeasures might also use this to store the position of the sensor which
/// created them
#[derive(Component, Debug, Deref, DerefMut, Clone)]
pub struct SensorPos(pub Vector2<f32>);

/// A position in the polar / sensor space, relative to a sensor
///
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

/// A position in the cartesian / kalman / absolute space
#[derive(Component, Debug, Default, Deref, DerefMut, Clone)]
pub struct Position(pub Vector2<f32>);
