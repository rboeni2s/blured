use glam::{Mat4, Vec2, Vec3, Vec4};
use spirv_std::{Sampler, image::Image2d, spirv};


#[spirv(vertex)]
pub fn vertex(
    #[spirv(uniform, descriptor_set = 1, binding = 0)] camera: &Mat4,
    pos: Vec3,
    tex: Vec2,
    #[spirv(position)] out_pos: &mut Vec4,
    #[spirv(location = 1)] out_tex: &mut Vec2,
)
{
    *out_tex = tex;
    *out_pos = camera * Vec4::new(pos.x, pos.y, pos.z, 1.0);
}


#[spirv(fragment)]
pub fn fragment(
    #[spirv(descriptor_set = 0, binding = 0)] texture: &Image2d,
    #[spirv(descriptor_set = 0, binding = 1)] sampler: &Sampler,
    #[spirv(uniform, descriptor_set = 2, binding = 0)] background: &Vec3,
    #[spirv(location = 0)] _pos: Vec4,
    #[spirv(location = 1)] mut tex: Vec2,
    out_color: &mut Vec4,
)
{
    tex.y = 1.0 - tex.y;

    if tex.x < 0.0
    {
        *out_color = Vec4::new(background.x, background.y, background.z, 1.0);
    }
    else
    {
        *out_color = texture.sample(*sampler, tex);
    }
}
