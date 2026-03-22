@group(0)
@binding(0)
var texture_diffuse: texture_2d<f32>;


@group(0)
@binding(1)
var sampler_diffuse: sampler;


@group(1)
@binding(0)
var<uniform> camera: mat4x4<f32>;


@group(2)
@binding(0)
var<uniform> background: vec4<f32>;



struct VertexInput
{
    @location(0) pos: vec3<f32>,
    @location(1) tex: vec2<f32>,
};


struct VertexOutput
{
    @builtin(position) pos: vec4<f32>,
    @location(0) tex: vec2<f32>,  
};


@vertex
fn vertex(vert: VertexInput) -> VertexOutput
{
    var out: VertexOutput;
    out.tex = vert.tex;
    out.pos = camera * vec4<f32>(vert.pos, 1.0);
    return out;
}


@fragment
fn fragment(vert: VertexOutput) -> @location(0) vec4<f32>
{
    var tex = vert.tex;

    if tex.x == -1.0
    {
        return background;
    }

    // flip y axis
    tex.y = 1 - tex.y;
    
    return textureSample(texture_diffuse, sampler_diffuse, tex);
}
 
