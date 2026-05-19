use bevy::prelude::*;

pub const DISP_SCALE: f32 = 0.03;
pub const TIME_SCALE: f32 = 1.;

#[derive(Component, Debug)]
pub struct Aircraft;

#[derive(Component, Debug)]
pub struct Sensor;

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
