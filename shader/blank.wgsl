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
var texture_diffuse: texture_2d<f32>;

@group(0)
@binding(1)
var sampler_diffuse: sampler;

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
    let size = vec2<f32>(textureDimensions(texture));
    var tex = vert.tex;

    // correct aspect ratio and center texture coordiantes
    tex = ((tex*size) - 0.5*size) / size.y;

    // draw the texture coordinates to the screen
    var color = vec3<f32>(tex, 0.0);

    return vec4<f32>(color, 1.0);
} 
