use bevy::{
    input::mouse::{AccumulatedMouseScroll, MouseScrollUnit},
    prelude::*,
};

pub fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

pub fn autozoom_camera(
    entities: Query<&Transform, (With<Mesh2d>, Without<Camera>)>,
    camera: Single<&mut Transform, With<Camera>>,
    proj: Single<&mut Projection, With<Camera>>,
    window: Single<&Window>,
) {
    const PADDING: f32 = 1.5;

    let mut pos = camera.into_inner();

    // figure out the area in which all entities are
    let min = entities.iter().fold(Vec2::splat(f32::MAX), |acc, e| {
        acc.min(e.translation.truncate())
    });
    let max = entities.iter().fold(Vec2::splat(f32::MIN), |acc, e| {
        acc.max(e.translation.truncate())
    });

    // position camera in the center of all entities
    let center = (min + max) * 0.5;
    pos.translation.x = center.x;
    pos.translation.y = center.y;

    let window_size = Vec2::new(window.width(), window.height());

    let size = max - min;
    let x_ratio = size.x / window_size.x;
    let y_ratio = size.y / window_size.y;

    if let Projection::Orthographic(ref mut orth_proj) = *proj.into_inner() {
        orth_proj.scale = f32::max(x_ratio, y_ratio) * PADDING;
    }
}

pub fn pan_camera(
    mut mouse_motion_events: MessageReader<CursorMoved>,
    mut camera: Single<&mut Transform, With<Camera>>,
    proj: Single<&Projection, With<Camera>>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
) {
    // only react if any of following three mouse buttons is pressed
    let react_to = [MouseButton::Left, MouseButton::Right, MouseButton::Middle];
    if !mouse_button_input.any_pressed(react_to) {
        return;
    }

    let orth = match *proj.into_inner() {
        Projection::Orthographic(ref x) => x,
        _ => unimplemented!(),
    };

    for event in mouse_motion_events.read() {
        let delta = match event.delta {
            Some(x) => x,
            None => continue,
        };

        let scale = orth.scale;
        camera.translation += Vec3::new(-delta.x * scale, delta.y * scale, 0.0);
    }
}

pub fn zoom_camera(
    proj: Single<&mut Projection, With<Camera>>,
    mouse_wheel_input: Res<AccumulatedMouseScroll>,
) {
    if mouse_wheel_input.delta.y == 0. {
        return;
    }

    let mut proj = proj.into_inner();
    let orth = match *proj {
        Projection::Orthographic(ref mut x) => x,
        _ => unimplemented!(),
    };

    // ensure the zoom on the web is the same as on desktop
    let normalized_delta = match mouse_wheel_input.unit {
        MouseScrollUnit::Line => mouse_wheel_input.delta.y,
        MouseScrollUnit::Pixel => {
            mouse_wheel_input.delta.y / MouseScrollUnit::SCROLL_UNIT_CONVERSION_FACTOR
        }
    };
    let zoom_speed = 0.5;
    let zoom_factor = (normalized_delta * zoom_speed).exp();

    orth.scale = (orth.scale / zoom_factor).clamp(0.1, 1000.);
}
