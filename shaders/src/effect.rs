use crate::*;


pub struct Effect<'a>
{
    pub params: &'a EffectParams,
    pub strength: f32,
    pub max_strength: f32,
    pub time: f32,
    pub size: Vec2,
    pub uv: Vec2,
    pub tex: Vec2,
}


impl<'a> Effect<'a>
{
    pub fn compute(
        self,
        i: &'a Image2d,
        s: &'a Sampler,
        f: impl FnOnce(Self, &'a Image2d, &'a Sampler) -> Vec3,
    ) -> Vec3
    {
        f(self, i, s)
    }
}


#[macro_export]
macro_rules! effect {
    ($func:expr) => {
        #[spirv(fragment)]
        pub fn fragment(
            #[spirv(descriptor_set = 0, binding = 0)] texture: &Image2d,
            #[spirv(descriptor_set = 0, binding = 1)] sampler: &Sampler,
            #[spirv(uniform, descriptor_set = 1, binding = 0)] params: &EffectParams,
            #[spirv(uniform, descriptor_set = 2, binding = 0)] EffectData {
                strength,
                max_strength,
                time,
            }: &EffectData,
            #[spirv(location = 0)] _pos: Vec4,
            #[spirv(location = 1)] mut tex: Vec2,
            color: &mut Vec4,
        )
        {
            // Get the texture dimensions
            let size = texture.query_size_lod::<glam::UVec2>(0).as_vec2();

            // Create centered and aspect ratio corrected uv coordinates.
            let uv = ((tex * size) - 0.5 * size) / size.y;

            // Flip texture coordinates horizontally
            tex.y = 1.0 - tex.y;

            let effect = Effect {
                params,
                strength: *strength,
                max_strength: *max_strength,
                time: *time,
                size,
                uv,
                tex,
            };

            *color = effect.compute(texture, sampler, $func).extend(1.0);
        }
    };
}
