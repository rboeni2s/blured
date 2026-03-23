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
var<uniform> effect_strength: vec2<f32>;


const TAU: f32 = 6.28318530718;
const DIR: f32 = 64.0;
const QUA: f32 = 20.0;


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
    // flip y axis
    var tex = vert.tex;
    tex.y = 1 - tex.y;

    let size = effect_strength.x;
    var color = textureSample(texture_diffuse, sampler_diffuse, tex);

    if size <= 0.0
    {
        return color;
    }
    
    let dims = textureDimensions(texture_diffuse);
    let radius = vec2<f32>(size, size) / vec2<f32>(dims);

    for (var d=0.0; d<TAU; d+=TAU/DIR)
    {
        for (var i=1.0/QUA; i<=1.0; i+=1.0/QUA)
        {
            let uv = tex + vec2<f32>(cos(d), sin(d)) * radius * i;
            color += textureSample(texture_diffuse, sampler_diffuse, uv);    
        }
    }

    return color / (QUA * DIR - (250.0 * (size/effect_strength.y)));
}
 
