use glam::{Vec3, Vec4};


#[cfg(target_arch = "spirv")]
pub use spirv_std::num_traits::Float;


#[repr(C)]
pub struct EffectParams
{
    pub(crate) param_a: Vec4,
    pub(crate) param_b: Vec4,
    pub(crate) param_c: Vec4,
}


#[repr(C)]
pub struct EffectData
{
    pub(crate) strength: f32,
    pub(crate) max_strength: f32,
    pub(crate) time: f32,
}


pub fn saturate(x: f32) -> f32
{
    x.clamp(0.0, 1.0)
}


pub fn pow(v: Vec3, power: f32) -> Vec3
{
    Vec3::new(v.x.powf(power), v.y.powf(power), v.z.powf(power))
}


pub fn exp(v: Vec3) -> Vec3
{
    Vec3::new(v.x.exp(), v.y.exp(), v.z.exp())
}


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


pub fn mix(x: Vec3, y: Vec3, a: f32) -> Vec3
{
    x * (1.0 - a) + y * a
}


pub fn smin(a: f32, b: f32, blend: f32) -> f32
{
    let h = f32::max(blend - f32::abs(a - b), 0.0);
    f32::min(a, b) - h * h / (blend * 4.0)
}
