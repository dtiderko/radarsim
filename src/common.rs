use bevy::prelude::*;

#[derive(Resource)]
pub struct SimTime(pub f32);

#[derive(Resource)]
pub struct RadarSweepCounter(pub u64);

#[derive(Component)]
pub struct CartesianMeasure;

#[derive(Component)]
pub struct PolarMeasure;

#[derive(Component, Debug)]
pub struct Aircraft;

#[derive(Component, Debug)]
pub struct Sensor;

#[derive(Component, Debug, Default)]
pub struct PolarPosition {
    pub range: f32,
    pub azimuth: f32,
}

#[derive(Component, Debug, Default)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Component, Debug, Default)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

#[derive(Component, Debug, Default)]
pub struct Acceleration {
    pub x: f32,
    pub y: f32,
}
