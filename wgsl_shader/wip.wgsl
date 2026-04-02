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

    // These are the uv coordinates (like texture coordinates)
    // but centered and aspect ratio corrected.
    // The uv coordinates span between -1 and 1 on each axis where 0 is the screen center
    let uv = ((vert.tex*size) - 0.5*size) / size.y;

    return effect(uv, tex, size);
}


const HIT: f32 = 0.0001;
const SKY: f32 = 20.0;
const NORMAL_ACC: vec2<f32> = vec2(0.0001, 0.0);


struct Ray
{
    ori: vec3<f32>,
    dir: vec3<f32>,
}


fn shoot(ray: Ray, t: f32) -> vec3<f32>
{
    return ray.ori + t * ray.dir;
}


fn march(ray: Ray) -> f32
{
    var t = 0.0;
    for (var i=0; i<100; i+=1)
    {
        let pos = shoot(ray, t);
        let dist = map(pos);

        if (dist < HIT)
        {
            break;
        }

        t += dist;
        if (t > SKY)
        {
            return -1.0;
        }
    }

    return t;
}


fn calc_normal(pos: vec3<f32>) -> vec3<f32>
{
    return normalize(vec3(
        map(pos + NORMAL_ACC.xyy) - map(pos - NORMAL_ACC.xyy),
        map(pos + NORMAL_ACC.yxy) - map(pos - NORMAL_ACC.yxy),
        map(pos + NORMAL_ACC.yyx) - map(pos - NORMAL_ACC.yyx),
    ));
}


fn smin(a: f32, b: f32, blend: f32) -> f32
{
    let h = max(blend - abs(a - b), 0.0);
    return min(a, b) - h*h / (blend*4.0);
}


fn ellipse_sdf(pos: vec3<f32>, radius: vec3<f32>) -> f32
{
    let pr = pos / radius;
    let d0 = length(pr);
    let d1 = length(pr / radius);
    return d0 * (d0-1.0) / d1;
    
}


fn sphere_sdf(pos: vec3<f32>, radius: f32) -> f32
{
    return length(pos) - radius;
    
}


fn camera(uv: vec2<f32>, pos: vec2<f32>) -> Ray
{
    let angle = effect_data.time * 0.1;
    let origin = vec3(sin(angle)*pos.x, pos.y, cos(angle)*pos.x);
    let lookat = vec3(0.0, 1.1, 0.0);
    let ww = normalize(lookat - origin);
    let uu = normalize(cross(ww, vec3(0.0, 1.0, 0.0)));
    let vv = normalize(cross(uu, ww));
    let direction = normalize(uv.x*uu + uv.y*vv + 1.5*ww);

    return Ray(origin, direction);
}


fn monster_sdf(pos: vec3<f32>) -> f32
{
    // let t = fract(effect_data.time * 0.8);
    let t = 0.5;

    // Animate the y_pos of the monster
    let y_pos = 4.0 * t * (1.0 - t);
    let center = vec3(0.0, y_pos - 0.125, 0.0);

    // Animate thte monster stetching while jumping
    let y_stretch = (0.5 + 0.55 * y_pos) * 0.8;
    let z_stretch = 0.5 / y_stretch;
    let x_stretch = 0.5 / y_stretch;
    let stretch = vec3(x_stretch, y_stretch, z_stretch);

    // New uv's based on y_pos curve
    let d_y_pos = 4.0 * (1.0 - 2.0*t);
    let u = vec2(1.0, d_y_pos);
    let v = vec2(-d_y_pos, 1.0);

    // Move the monster using the new uv's
    var init_pos = pos-center;
    let init_pos_yz = vec2(dot(u, init_pos.yz), dot(v, init_pos.yz));
    // init_pos.y = init_pos_yz.x;
    // init_pos.z = init_pos_yz.y;


    let belly_sdf = ellipse_sdf(init_pos, vec3(0.25));
    let head_sdf = ellipse_sdf(init_pos + vec3(0.0, -0.28, 0.0), vec3(0.2));
    let backhead_sdf = ellipse_sdf(init_pos + vec3(0.0, -0.28, 0.1), vec3(0.2));
    let eye_l = sphere_sdf(init_pos + vec3(-0.1, -0.3, -0.14), 0.05);
    let eye_r = sphere_sdf(init_pos + vec3(0.1, -0.3, -0.14), 0.05);

    var monster = smin(belly_sdf, head_sdf, 0.1);
    monster = smin(monster, backhead_sdf, 0.03);
    monster = smin(monster, eye_l, 0.002);
    monster = smin(monster, eye_r, 0.002);

    return monster;
}


fn map(pos: vec3<f32>) -> f32
{
    let monster = monster_sdf(pos);
    let plane_sdf = pos.y + 0.25;
    return min(monster, plane_sdf);
}


fn effect(uv: vec2<f32>, tex: vec2<f32>, size: vec2<f32>) -> vec4<f32>
{
    var color = vec3(0.0);

    let sun_dir = normalize(vec3(0.8, 0.55, 0.2));
    let matt = vec3(0.18);

    // Setup a camera and march a ray from it...
    var ray = camera(uv, vec2(3.5, 0.9));
    let t = march(ray);
    
    // Check if the ray hit "something"
    if (t > 0.0)
    {
        // Calculate the normals of "something"
        let pos = shoot(ray, t);
        let normal = calc_normal(pos);

        // March shadow rays
        var shadow_ray = Ray(pos + (normal * HIT), sun_dir);
        let shadow_dist = march(shadow_ray);

        // Calculate lighting
        let sun_light = clamp(dot(normal, sun_dir), 0.0, 1.0);
        let sun_shadow = step(shadow_dist, 0.0);
        let sky_light = clamp(0.5 + 0.5 * dot(normal, vec3(0.0, 1.0, 0.0)), 0.0, 1.0);
        let bounce_light = clamp(0.5 + 0.5 * dot(normal, vec3(0.0, -1.0, 0.0)), 0.0, 1.0);

        // Apply lighting
        color += matt * vec3(7.0, 5.0, 3.0) * sun_light * sun_shadow;
        color += matt * vec3(0.5, 0.8, 0.9) * sky_light;
        color += matt * vec3(0.7, 0.3, 0.2) * bounce_light;
    }
    else
    {
        color = vec3(0.2, 0.6, 1.0) - max(ray.dir.y, 0.0) * 0.5;
        color = mix(color, vec3(0.7, 0.75, 0.8), exp(-10.0 * ray.dir.y));
    }

    return vec4(pow(color, vec3(0.4545)), 0.0);
}
