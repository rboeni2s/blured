#![no_std]

// Modules for each effect
//////////////////////////
pub mod blank;
pub mod jumping;


// Shared imports, data and functions for all effects as well as a common vertex entry point
////////////////////////////////////////////////////////////////////////////////////////////
pub mod common;
pub mod effect;
pub mod scene;
pub use common::*;
pub use effect::Effect;
pub use glam::{Vec2, Vec3, Vec4, swizzles::*};
pub use spirv_std::{Sampler, image::Image2d, spirv};


#[spirv(vertex)]
pub fn vertex(
    pos: Vec3,
    tex: Vec2,
    #[spirv(position)] out_pos: &mut Vec4,
    #[spirv(location = 1)] out_tex: &mut Vec2,
)
{
    *out_tex = tex;
    *out_pos = Vec4::new(pos.x, pos.y, pos.z, 1.0);
}
