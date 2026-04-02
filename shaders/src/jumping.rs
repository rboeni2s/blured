#![allow(unused)]


use crate::*;


const HIT: f32 = 0.0001;
const SKY: f32 = 20.0;


struct Ray
{
    ori: Vec3,
    dir: Vec3,
}


impl Ray
{
    fn new(ori: Vec3, dir: Vec3) -> Self
    {
        Self { ori, dir }
    }

    fn shoot(&self, t: f32) -> Vec3
    {
        self.ori + t * self.dir
    }

    fn march(&self, map: impl Fn(Vec3) -> f32) -> Option<f32>
    {
        let mut t = 0.0;

        loop
        {
            let pos = self.shoot(t);
            let dist = map(pos);

            if dist < HIT
            {
                return Some(t);
            }
            else if t > SKY
            {
                return None;
            }

            t += dist;
        }
    }

    fn camera(uv: Vec2, time: f32) -> Self
    {
        let pos = Vec2::new(3.5, 0.9);
        let look_at = Vec3::new(0.0, 1.1, 0.0);

        let angle = time * 0.1;
        let origin = Vec3::new(angle.sin() * pos.x, pos.y, angle.cos() * pos.x);
        let ww = (look_at - origin).normalize();
        let uu = ww.cross(Vec3::new(0.0, 1.0, 0.0)).normalize();
        let vv = uu.cross(ww).normalize();
        let direction = (uv.x * uu + uv.y * vv + 1.5 * ww).normalize();

        return Self::new(origin, direction);
    }
}


fn calc_normal(pos: Vec3, map: impl Fn(Vec3) -> f32) -> Vec3
{
    const NORMAL_ACC: Vec2 = Vec2::new(0.0001, 0.0);

    Vec3::new(
        map(pos + NORMAL_ACC.xyy()) - map(pos - NORMAL_ACC.xyy()),
        map(pos + NORMAL_ACC.yxy()) - map(pos - NORMAL_ACC.yxy()),
        map(pos + NORMAL_ACC.yyx()) - map(pos - NORMAL_ACC.yyx()),
    )
    .normalize()
}


fn smin(a: f32, b: f32, blend: f32) -> f32
{
    let h = f32::max(blend - f32::abs(a - b), 0.0);
    return f32::min(a, b) - h * h / (blend * 4.0);
}


fn ellipse_sdf(pos: Vec3, radius: Vec3) -> f32
{
    let pr = pos / radius;
    let d0 = pr.length();
    let d1 = (pr / radius).length();
    return d0 * (d0 - 1.0) / d1;
}


fn sphere_sdf(pos: Vec3, radius: f32) -> f32
{
    return pos.length() - radius;
}


fn monster_sdf(pos: Vec3, time: f32) -> f32
{
    let t = (time * 0.8).fract();
    let t = 0.5;

    // Animate the y_pos of the monster
    let y_pos = 4.0 * t * (1.0 - t);
    let center = Vec3::new(0.0, y_pos - 0.125, 0.0);

    // Animate the monster stretching while jumping
    let y_stretch = (0.5 + 0.55 * y_pos) * 0.8;
    let z_stretch = 0.5 / y_stretch;
    let x_stretch = 0.5 / y_stretch;
    let stretch = Vec3::new(x_stretch, y_stretch, z_stretch);

    // New uv's based on y_pos curve
    let d_y_pos = 4.0 * (1.0 - 2.0 * t);
    let u = Vec2::new(1.0, d_y_pos);
    let v = Vec2::new(-d_y_pos, 1.0);

    // Move the monster using the new uv's
    let mut init_pos = pos - center;
    // let init_pos_yz = Vec2::new(u.dot(init_pos.yz()), v.dot(init_pos.yz()));
    // init_pos.y = init_pos_yz.x;
    // init_pos.z = init_pos_yz.y;

    let belly_sdf = ellipse_sdf(init_pos, Vec3::splat(0.25));
    let head_sdf = ellipse_sdf(init_pos + Vec3::new(0.0, -0.28, 0.0), Vec3::splat(0.2));
    let head_back_sdf = ellipse_sdf(init_pos + Vec3::new(0.0, -0.28, 0.1), Vec3::splat(0.2));
    let eye_l = sphere_sdf(init_pos + Vec3::new(-0.1, -0.3, -0.14), 0.05);
    let eye_r = sphere_sdf(init_pos + Vec3::new(0.1, -0.3, -0.14), 0.05);

    let mut monster = smin(belly_sdf, head_sdf, 0.1);
    monster = smin(monster, head_back_sdf, 0.03);
    monster = smin(monster, eye_l, 0.002);
    monster = smin(monster, eye_r, 0.002);

    return monster;
}


fn map(pos: Vec3, time: f32) -> f32
{
    let monster = monster_sdf(pos, time);
    let plane_sdf = pos.y + 0.25;
    return f32::min(monster, plane_sdf);
}


effect!(|Effect { uv, time, .. }, _, _| {
    let mut color = Vec3::ZERO;
    let sun_dir = Vec3::new(0.8, 0.55, 0.2).normalize();
    let matt = Vec3::splat(0.18);

    // Setup a camera and march a ray from it...
    let ray = Ray::camera(uv, time);

    match ray.march(|pos| map(pos, time))
    {
        Some(dist) =>
        {
            // Calculate the normals of "something"
            let pos = ray.shoot(dist);
            let normal = calc_normal(pos, |p| map(p, time));

            // March shadow rays
            let shadow_ray = Ray::new(pos + (normal * HIT), sun_dir);
            let shadow_dist = shadow_ray.march(|pos| map(pos, time));

            // Calculate lighting
            let sun_light = f32::clamp(normal.dot(sun_dir), 0.0, 1.0);
            let sun_shadow = shadow_dist.map_or(1.0, |_| 0.0);
            let sky_light = f32::clamp(0.5 + 0.5 * normal.dot(Vec3::new(0.0, 1.0, 0.0)), 0.0, 1.0);
            let bounce_light =
                f32::clamp(0.5 + 0.5 * normal.dot(Vec3::new(0.0, -1.0, 0.0)), 0.0, 1.0);

            // Apply lighting
            color += matt * Vec3::new(7.0, 5.0, 3.0) * sun_light * sun_shadow;
            color += matt * Vec3::new(0.5, 0.8, 0.9) * sky_light;
            color += matt * Vec3::new(0.7, 0.3, 0.2) * bounce_light;
        }

        None =>
        {
            // Draw a sky
            color = Vec3::new(0.2, 0.6, 1.0) - ray.dir.y.max(0.0) * 0.5;
            color = mix(
                color,
                Vec3::new(0.7, 0.75, 0.8),
                f32::exp(-10.0 * ray.dir.y),
            );
        }
    }

    return pow(color, 0.4545);
});
