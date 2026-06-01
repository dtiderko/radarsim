#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> color: vec4<f32>;

// because else we would only show normal distribution from sqrt(2) to sqrt(2)
// which is only the tip of the bell
const SCALING = 4.0;

fn normd(x: f32) -> f32 {
    return exp(-pow(x * SCALING, 2) / 2.) / sqrt(radians(360.));
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    // Map UV coordinates from [0, 1] to [-1, 1] to get distance from center
    let local_pos = mesh.uv * 2.0 - 1.0;
    let distance = dot(local_pos, local_pos);

    return vec4<f32>(color.rgb, normd(distance));
}
