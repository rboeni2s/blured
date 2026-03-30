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


// Defaults:
// const SCALE: f32 = 60.0;
// const SPEED: f32 = 0.09;
// const BACKGROUND: vec3<f32> = vec3(0.001, 0.001, 0.02);
struct EffectParams
{
    // split param_a into scale and speed and a vec2 for padding
    // param_a: vec4<f32>,
    scale: f32,
    speed: f32,
    padding: vec2<f32>,

    // use param_b as background colo
    // param_b: vec4<f32>,
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

    return effect(uv, tex);
}


/// Uses a dot mask to mask out a dotted version of the background image
fn effect(uv: vec2<f32>, tex: vec2<f32>) -> vec4<f32>
{
    // Get the texture color at tex
    var color = textureSample(texture, tsampler, tex);

    // Make the dots rotate by rotating the uv coordinates of the mask
    let speed = effect_data.time * effect_params.speed;
    let s = sin(speed);
    let c = cos(speed);
    let rot_uv = uv * mat2x2(c, -s, s, c);
    
    // Calculate the mask
    let mask = dot_mask(rot_uv);

    // Apply the dot mask to the texture 
    color = color * mask;

    // Put the baclground color everywhere where no dots are
    if mask == 0.0
    {
        color = vec4(effect_params.background.xyz, 1.0);
    }

    return color;
}


/// Masks a grid of dots
fn dot_mask(uv: vec2<f32>) -> f32
{
    let grid_uv = fract(uv * effect_params.scale) - 0.5;
    return smoothstep(0.4, 0.3, length(grid_uv));
}
