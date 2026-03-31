struct VertexInput
{
    @location(0) pos: vec3<f32>,
    @location(1) tex: vec2<f32>,
}


struct VertexOutput
{
    @builtin(position) pos: vec4<f32>,
    @location(0) tex: vec2<f32>,  
}


struct EffectParams
{
    param_a: vec4<f32>,
    param_b: vec4<f32>,
    background: vec4<f32>,
    param_c: vec4<f32>,
}


struct EffectData
{
    strength: f32,
    max_strength: f32,
    time: f32,
}


@group(0)
@binding(0)
var texture: texture_2d<f32>;

@group(0)
@binding(1)
var tsampler: sampler;

@group(1)
@binding(0)
var<uniform> effect_params: EffectParams;

@group(2)
@binding(0)
var<uniform> effect_data: EffectData;


const PI: f32 = 3.14159265359;
const TAU: f32 = 2.0 * PI;


@vertex
fn vertex(vert: VertexInput) -> VertexOutput
{
    var out: VertexOutput;
    out.tex = vert.tex;
    out.pos = vec4<f32>(vert.pos, 1.0);
    return out;
}


@fragment
fn fragment(vert: VertexOutput) -> @location(0) vec4<f32>
{
    // A vec2 of the surface size in pixels
    let size = vec2<f32>(textureDimensions(texture));

    // These are the texture cooardinates use these for texture sampling
    let tex = vec2(vert.tex.x, 1 - vert.tex.y);

    // These are the uv cooadinates (like texture cooadinates)
    // but centered and aspect ratio corrected.
    // The uv coordinates span between -1 and 1 on each axis where 0 is the screen center
    let uv = ((vert.tex*size) - 0.5*size) / size.y;

    return effect(uv, tex, size);
}


fn effect(uv_in: vec2<f32>, tex: vec2<f32>, size: vec2<f32>) -> vec4<f32>
{
    return textureSample(texture, tsampler, tex);
}
