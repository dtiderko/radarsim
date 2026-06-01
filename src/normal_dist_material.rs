use bevy::{
    prelude::*,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
    sprite_render::{AlphaMode2d, Material2d},
};

const SHADER_PATH: &str = "shaders/normal_dist_material.wgsl";

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct NormalDistMaterial {
    #[uniform(0)]
    pub color: LinearRgba,
}
impl Material2d for NormalDistMaterial {
    fn fragment_shader() -> ShaderRef {
        SHADER_PATH.into()
    }

    // declare this material as a transparent blend
    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}
