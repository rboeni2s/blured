use core::f32::consts::{PI, TAU};

use crate::*;
use raymarch::*;


type Sdf = crate::raymarch::Sdf<Mat>;


material!(Mat => [Metal, Normal]);
effect!(|Effect { uv, time, size, .. }, texture, sampler| {
    raymarcher(uv, time, texture, sampler, size)
});


fn gear_tooth_sdf(pos: Vec3) -> Sdf
{
    // Segment the uv space into 12 sectors to mirror one tooth into all of them
    let angle = TAU / 12.0;
    let angle = (pos.z.atan2(pos.x) / angle).round() * angle;

    let pos = {
        let (sin, cos) = angle.sin_cos();
        Vec3::new(pos.x * cos + pos.z * sin, pos.y, pos.x * -sin + pos.z * cos)
    };

    let tooth = sdf(pos, |p| box_sdf(p, Vec3::new(0.045, 0.05, 0.02)))
        .pos(0.19, 0.0, 0.0)
        .round(0.007);

    tooth.build()
}


fn gear_cross_sdf(pos: Vec3) -> Sdf
{
    let mut pos = (pos - Vec3::new(0.0, 0.03, 0.0)).abs();
    pos = if pos.z > pos.x { pos.zyx() } else { pos.xyz() };
    sdf(pos, |p| (p.yz().length() - 0.005).max(p.x.abs() - 0.15)).build()
}


fn double_gear_sdf(mut pos: Vec3, time: f32, mat: Mat) -> Sdf
{
    pos = pos.rotate_y((time * 0.3).fract() * TAU);
    pos.y = pos.y.abs();

    let scale = 0.105;
    let go = Vec3::new(0.0, 0.46 + scale, 0.0);

    gear_tooth_sdf(pos - go)
        .mat(mat)
        .join(
            sdf(pos - go, |p| pipe_sdf(p, 0.18, 0.05, 0.015)).mat(mat),
            0.005,
        )
        .clip(
            sdf(pos, |p| (p.length() - (0.5 + scale)).abs() - 0.03),
            0.0021,
        )
        .join(gear_cross_sdf(pos - go), 0.002)
        .join(
            sdf(pos, |p| cylinder_sdf(p, 0.005, 0.49 + scale)).pos(0.0, 0.0, 0.0),
            0.003,
        )
}


fn gear_ball_sdf(pos: Vec3, time: f32) -> Sdf
{
    // Segment and mirror the uv's to create multiple double gears
    let (pos, time) = {
        let p = pos.abs();
        if p.x > p.y && p.x > p.z
        {
            (pos.yxz(), time)
        }
        else if p.z > p.y
        {
            (pos.xzy(), time)
        }
        else
        {
            (pos, time)
        }
    };

    double_gear_sdf(pos, time, Mat::Metal)
}


fn rot45(p: Vec2) -> Vec2
{
    Vec2::new(p.x + p.y, p.x - p.y) * 0.707107
}


fn map(pos: Vec3, time: f32) -> Sdf
{
    let time_scale = 1.0;
    let rot_pos1 = rot45(pos.zy()).extend(pos.x);
    let rot_pos2 = rot45(pos.yx()).extend(pos.z);
    let rot_pos3 = rot45(pos.xz()).extend(pos.y);
    let rot_pos4 = rot45(pos.zx()).extend(pos.y);

    let time_off = (time * time_scale) + (0.13 / time_scale);

    gear_ball_sdf(pos, time * time_scale)
        .join_sharp(double_gear_sdf(rot_pos1, time_off, Mat::Metal))
        .join_sharp(double_gear_sdf(rot_pos2, -time_off, Mat::Normal))
        .join_sharp(double_gear_sdf(rot_pos3, time_off, Mat::Metal))
        .join_sharp(double_gear_sdf(rot_pos4, time_off, Mat::Metal))
        .join_sharp(sdf(pos, |p| sphere_sdf(p, 0.2)))
}


fn raymarcher(uv: Vec2, time: f32, texture: &Image2d, sampler: &Sampler, size: Vec2) -> Vec3
{
    let color = antialiase(2, uv, |uv| {
        let camera = Ray::camera(uv, Vec3::ZERO, 3.0, 0.8, time * 0.1);

        match camera.march(|p| map(p, time))
        {
            (MarchResult::Hit, obj) => shade(obj, camera, time, texture, sampler, size),
            (MarchResult::Miss, _) => Vec3::splat(1.0 + 1.8 * uv.y) * 0.0001,
        }
    });

    finalize(uv, color)
}


fn shade(obj: Sdf, cam: Ray, time: f32, texture: &Image2d, sampler: &Sampler, size: Vec2) -> Vec3
{
    let hit_pos = cam.shoot(obj.dist);
    let normals = calc_normal(hit_pos, |p| map(p, time));
    let reflect = cam.dir.reflect(normals);


    let color = match obj.mat
    {
        Mat::Metal =>
        {
            let tex_scale = 3.0;

            let matt = 0.5
                * texture
                    .sample(*sampler, (obj.pos.xy() * tex_scale) + 0.5)
                    .xyz()
                + 0.5
                    * texture
                        .sample(*sampler, (obj.pos.yz() * tex_scale) + 0.5)
                        .yxz();
            matt * 0.22
        }

        Mat::Normal => 0.5 + 0.5 * normals,
    };

    let mut focc = (0.5 + 0.5 * normals.dot(hit_pos.normalize())).clamp(0.003, 1.0);
    focc += 0.1 + 0.9 * (hit_pos.length() / 0.535).clamp(0.0, 1.0);
    let ao = calc_occlusion(hit_pos, normals, |p| map(p, time)) * focc;


    let diffuse_light = 0.5 + 0.5 * normals.y;

    let fres = (1.0 + cam.dir.dot(normals)).clamp(0.0, 1.0) * 2.0;

    let specular_light = {
        let spec = smoothstep(0.5, 0.6, reflect.y);
        spec * (color + (1.0 - color) * fres.powf(5.0)) * 6.0
    };

    let mut light = Vec3::splat(0.0);
    light += 0.08 * Vec3::new(0.7, 0.8, 1.1) * diffuse_light * ao * fres;
    light += 1.0 * Vec3::new(0.7, 0.8, 1.1) * specular_light * diffuse_light;

    color * light
}
