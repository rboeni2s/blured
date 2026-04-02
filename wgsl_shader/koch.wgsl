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


const SPEED: f32 = 1.0;
const LINE: f32 = 0.3;

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
const D45: f32 = PI * (2.0 / 3.0);
const D120: f32 = PI * (5.0 / 6.0);


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


/// Reflects a point around a line.
///
/// Reflect point `p` around the line defined by `origin` and `angle`.
fn mirror(origin: vec2<f32>, angle: f32, p: vec2<f32>) -> vec2<f32>
{
    let refl = vec2(sin(angle), cos(angle));
    let dist = dot(p+origin, refl);
    return p - (refl * min(0.0, dist) * 2.0);
}


/// Reflects a point around a line in reverse.
///
/// Reflect point `p` around the line defined by `origin` and `angle`.
fn mirror_rev(origin: vec2<f32>, angle: f32, p: vec2<f32>) -> vec2<f32>
{
    let refl = vec2(sin(angle), cos(angle));
    let dist = dot(p+origin, refl);
    return p - (refl * max(0.0, dist) * 2.0);
}




fn effect(uv_in: vec2<f32>, tex: vec2<f32>, size: vec2<f32>) -> vec4<f32>
{
    var color = vec3(0.0);
    var uv = uv_in * 1.5;

    // Position the fractal
    uv.y += tan(D120) * 0.5;

    // Mirror all bends
    uv.x = abs(uv.x);
    uv = mirror_rev(vec2(-0.5, 0.0), D120, uv);

    // Move the uv to the left by 3*0.5=1.5 because the first
    // scaling will move the uv's 1.5 to the right 
    uv.x += 0.5;

    // Fold the uv's a few times to create the bends
    var compression = 1.0;
    for (var i=0; i<4; i+=1)
    {
        // Scale the uv coords
        uv *= 3.0;
        compression *= 3.0; // keep track of how compressed the uv space is
        uv.x -= 1.5;

        // Mirror the uv around the vertical center.
        uv = vec2(abs(uv.x) - 0.5, uv.y);

        // Reflect the uv coords to create a bend in the line 
        uv = mirror(vec2(0.0), D45, uv);
    }

    // Draw a line, even though it is a straight line it will appear to have bends because of the uv's.
    let line_dist = length(uv - vec2(clamp(uv.x, -1.0, 1.0), 0));
    var line = 1.0 - smoothstep(1.0/ size.y, 0.02 * LINE, line_dist / compression);
    line += 1.0 - smoothstep(1.0/ size.y, 0.01 * LINE, line_dist / compression);

    // Apply the compression to the uv's magnitude aswell
    uv /= compression;

    // color += vec3(uv, 0.0);
    color += textureSample(texture, tsampler, uv * 0.6 + abs(sin(effect_data.time * 0.02 * SPEED)) * 0.7).xyz;

    if line > 0.0
    {
        color = (vec3(1.0)-color) * line + color * (1.0 - line);
    }

    return vec4(color, 1.0);
}
