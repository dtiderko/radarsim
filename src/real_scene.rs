use bevy::prelude::*;

const DISP_SCALE: f32 = 0.01;
const TIME_SCALE: f32 = 100.;

#[derive(Resource)]
struct SimTime(f32);

#[derive(Component)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Component)]
struct Aircraft;

pub struct RealScene;
impl Plugin for RealScene {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_scene)
            .add_systems(Update, (move_aircraft, update_render).chain());
    }
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.insert_resource(SimTime(0.));

    let shape = meshes.add(Circle::new(5.0));
    let color = Color::srgb(1., 0., 0.);
    let pos = Position { x: 0.0, y: 0.0 };

    commands.spawn((
        Aircraft,
        // render
        Mesh2d(shape),
        MeshMaterial2d(materials.add(color)),
        Transform::from_xyz(pos.x, pos.y, 0.),
        // data
        pos,
    ));
}

fn move_aircraft(
    time: Res<Time>,
    mut sim_time: ResMut<SimTime>,
    mut query: Query<&mut Position, With<Aircraft>>,
) {
    const V: f32 = 300.;
    const Q: f32 = 9.;

    let a = V.powi(2) / Q;
    let w = Q / (2. * V);

    sim_time.0 += time.delta_secs() * TIME_SCALE;

    for mut spos in &mut query {
        spos.x = a * f32::sin(w * sim_time.0);
        spos.y = a * f32::sin(2. * w * sim_time.0);
    }
}

fn update_render(mut query: Query<(&Position, &mut Transform), With<Aircraft>>) {
    for (sim_pos, mut render_pos) in &mut query {
        render_pos.translation[0] = sim_pos.x * DISP_SCALE;
        render_pos.translation[1] = sim_pos.y * DISP_SCALE;
    }
}
