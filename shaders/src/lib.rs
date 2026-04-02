#![no_std]

// Modules for each effect
//////////////////////////
pub mod blank;
pub mod jumping;


// Shared imports, data and functions for all effects as well as a common vertex entry point
////////////////////////////////////////////////////////////////////////////////////////////
pub mod scene;
pub use glam::{Vec2, Vec3, Vec4, vec3};
pub use spirv_std::{Sampler, image::Image2d, spirv};

#[cfg(target_arch = "spirv")]
pub use spirv_std::num_traits::Float;


#[repr(C)]
pub struct EffectParams
{
    param_a: Vec4,
    param_b: Vec4,
    param_c: Vec4,
}


#[repr(C)]
pub struct EffectData
{
    strength: f32,
    max_strength: f32,
    time: f32,
}


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


pub fn saturate(x: f32) -> f32
{
    x.clamp(0.0, 1.0)
}


pub fn pow(v: Vec3, power: f32) -> Vec3
{
    vec3(v.x.powf(power), v.y.powf(power), v.z.powf(power))
}


pub fn exp(v: Vec3) -> Vec3
{
    vec3(v.x.exp(), v.y.exp(), v.z.exp())
}


/// Based on: <https://seblagarde.wordpress.com/2014/12/01/inverse-trigonometric-functions-gpu-optimization-for-amd-gcn-architecture/>
pub fn acos_approx(v: f32) -> f32
{
    let x = v.abs();
    let mut res = -0.155972 * x + 1.56467; // p(x)
    res *= (1.0f32 - x).sqrt();

    if v >= 0.0
    {
        res
    }
    else
    {
        core::f32::consts::PI - res
    }
}


pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32
{
    // Scale, bias and saturate x to 0..1 range
    let x = saturate((x - edge0) / (edge1 - edge0));
    // Evaluate polynomial
    x * x * (3.0 - 2.0 * x)
}
