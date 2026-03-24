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
    let size = vec2<f32>(textureDimensions(texture));
    var tex = vert.tex;
    let uv = vec2(tex.x, 1.0-tex.y);

    // correct aspect ratio and center texture coordiantes
    tex = ((tex*size) - 0.5*size) / size.y;
    return vec4<f32>(effect(tex), 1.0) * textureSample(texture, tsampler, uv);
}


/// Returns the distance from point p to a line segment spanning from a to b.
fn line_sdf(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>) -> f32
{
    let pa = p - a;
    let ba = b - a;
    let t = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
    return length(pa - ba * t);
}


/// Generates a pseudo random f32 seeded by p
fn rand1(p: vec2<f32>) -> f32
{
    var n = fract(p * vec2(239.39, 812.93));
    n += dot(n, n + 39.12);
    return(fract(n.x * n.y));
}


/// Generates a peudo random vec2 seedd by p
fn rand2(p: vec2<f32>) -> vec2<f32>
{
    let x = rand1(p);
    let y = rand1(p+x);
    return vec2(x, y);
}


const SCALE: f32 = 15.0;
const SPEED: f32 = 0.4;
const DIM: f32 = 21.0;
const AMBIENT: f32 = 0.001;


// Draws a line between a and b this line will fade based on its length
fn line(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>) -> vec3<f32>
{
    let sdf = line_sdf(p, a, b);
    let line = vec3(smoothstep(0.01, 0.005, sdf));
    return line * smoothstep(1.2, 0.8, length(a-b));
}


/// Returns a random, animated point in grid_cell
/// A smaller dist will cause the point to be closer to its cells center
fn grid_point(grid_cell: vec2<f32>, tex_grid: vec2<f32>, dist: f32) -> vec2<f32>
{
    let noise = rand2(grid_cell);
    return sin(noise * (effect_data.time*SPEED + 5.0)) * dist;
}


/// Draws the effect
fn effect(tex: vec2<f32>) -> vec3<f32> 
{
    var scale = SCALE;
    var color = vec3(AMBIENT);
    let tex_grid = fract(scale * tex) - 0.5;
    let grid_cell = floor(scale * tex);

    // put a random point in each cell
    let point_origin = grid_point(grid_cell, tex_grid, 0.39);

    // get the position of each neighbouring point, and draw a line to the neighbours
    var lines = vec3(0.0);
    var neighbours: array<vec2<f32>, 9>;
    var neighbour_index = 0;
    for (var y=-1.0; y<=1.0; y+=1)
    {
        for (var x=-1.0; x<=1.0; x+=1)
        {
            let offset = vec2(x, y);
            neighbours[neighbour_index] = offset + grid_point(grid_cell + offset, tex_grid, 0.39);

            // draw a line from the center point to its neighbour
            lines += line(tex_grid, point_origin, neighbours[neighbour_index]);

            // draw a light at the end of each line
            let light_dist = (neighbours[neighbour_index] - tex_grid) * DIM;
            lines += 1.0 / dot(light_dist, light_dist);
                        
            neighbour_index += 1;
        }
    }

    // fix interrupted lines
    lines += line(tex_grid, neighbours[1], neighbours[3]);
    lines += line(tex_grid, neighbours[1], neighbours[5]);
    lines += line(tex_grid, neighbours[7], neighbours[3]);
    lines += line(tex_grid, neighbours[7], neighbours[5]);

    color += lines;
  

    // draw the tex grid for debugging
    // if tex_grid.x > 0.49 || tex_grid.y > 0.49
    // {
    //     color = vec3(0.0, 1.0, 0.0);
    // }

    return color;
}
 
