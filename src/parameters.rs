
pub struct Parameters {
    pub shader_type: crate::shaders::provider::ShaderType,
    pub img_size_px: u16,
    pub max_iter: u8,
    pub limits: [f32; 4]
}
