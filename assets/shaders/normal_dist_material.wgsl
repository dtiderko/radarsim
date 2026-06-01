#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> color: vec4<f32>;

const PI = radians(180.);
const SIG = 0.15;

fn pdf(x: f32) -> f32 {
    const FRONT = 1. / sqrt(2 * PI * pow(SIG, 2));
    const EXP_LOWER = -2 * pow(SIG, 2);

    return FRONT * exp(pow(x, 2) / EXP_LOWER);
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    // Map UV coordinates from [0, 1] to [-1, 1] to get distance from center
    let local_pos = mesh.uv * 2.0 - 1.0;
    let distance_sq = dot(local_pos, local_pos);

    return vec4<f32>(color.rgb, pdf(distance_sq));
}
