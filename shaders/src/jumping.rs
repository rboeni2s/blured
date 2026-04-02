use crate::*;


#[spirv(fragment)]
pub fn fragment(
    #[spirv(descriptor_set = 0, binding = 0)] texture: &Image2d,
    #[spirv(descriptor_set = 0, binding = 1)] sampler: &Sampler,
    #[spirv(uniform, descriptor_set = 1, binding = 0)] _params: &EffectParams,
    #[spirv(uniform, descriptor_set = 1, binding = 0)] _data: &EffectData,
    #[spirv(location = 0)] _pos: Vec4,
    #[spirv(location = 1)] mut tex: Vec2,
    color: &mut Vec4,
)
{
    // Flip texture coordinates horizontally
    tex.y = 1.0 - tex.y;
    *color = texture.sample(*sampler, tex);
}
