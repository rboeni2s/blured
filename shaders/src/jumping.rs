#![allow(unused)]
use crate::*;
use raymarch::*;


// Define materials
material!(Mat => [MonsterBody, MonsterEye, Ground, Stone, MonsterPupil]);


fn monster_sdf(pos: Vec3, time: f32) -> Sdf<Mat>
{
    let t = (time * 0.8).fract();
    let t = 0.5; // Freeze the animation

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
    let mut pos = pos - center;
    // let init_pos_yz = Vec2::new(u.dot(init_pos.yz()), v.dot(init_pos.yz()));
    // init_pos.y = init_pos_yz.x;
    // init_pos.z = init_pos_yz.y;

    let belly = sdf(pos, |p| ellipse_sdf(p, Vec3::splat(0.25))).mat(Mat::MonsterBody);

    let head = sdf(pos, |p| ellipse_sdf(p, Vec3::splat(0.2)))
        .pos(0.0, 0.28, 0.0)
        .mat(Mat::MonsterBody)
        .build()
        .join(
            sdf(pos, |p| ellipse_sdf(p, Vec3::splat(0.2)))
                .pos(0.0, 0.28, -0.1)
                .mat(Mat::MonsterBody),
            0.03,
        );

    let eye_l = sdf(pos, |p| sphere_sdf(p, 0.05))
        .pos(0.1, 0.3, 0.14)
        .mat(Mat::MonsterEye)
        .build()
        .join(
            sdf(pos, |p| sphere_sdf(p, 0.033))
                .pos(0.11, 0.3, 0.16)
                .mat(Mat::MonsterPupil),
            0.002,
        );

    let eye_r = sdf(pos, |p| sphere_sdf(p, 0.05))
        .pos(-0.1, 0.3, 0.14)
        .mat(Mat::MonsterEye)
        .build()
        .join(
            sdf(pos, |p| sphere_sdf(p, 0.033))
                .pos(-0.11, 0.3, 0.16)
                .mat(Mat::MonsterPupil),
            0.002,
        );

    let eyelid_l = sdf(pos, |p| ellipse_sdf(p, Vec3::new(0.07, 0.035, 0.04)))
        .pos(0.12, 0.375, 0.14)
        .rot_z(22.0)
        .rot_y(-15.0)
        .mat(Mat::MonsterBody);

    let eyelid_r = sdf(pos, |p| ellipse_sdf(p, Vec3::new(0.07, 0.035, 0.04)))
        .pos(-0.12, 0.375, 0.14)
        .rot_z(-22.0)
        .rot_y(15.0)
        .mat(Mat::MonsterBody);

    let mouth = sdf(pos, |p| ellipse_sdf(p, Vec3::new(0.05, 0.04, 0.18))).pos(0.0, 0.2, 0.09);

    belly
        .build()
        .join(head, 0.1)
        .join(eyelid_l, 0.08)
        .join(eyelid_r, 0.08)
        .join(eye_l, 0.002)
        .join(eye_r, 0.002)
        .carve(mouth, 0.03)
}


fn map(pos: Vec3, time: f32) -> Sdf<Mat>
{
    let monster = monster_sdf(pos, time);

    let stone = sdf(pos, |p| ellipse_sdf(p, Vec3::new(0.4, 0.2, 0.25)))
        .pos(-1.0, -0.25, -1.0)
        .mat(Mat::Stone);

    let ground = sdf(pos, plane_sdf).pos(0.0, -0.25, 0.0).mat(Mat::Ground);

    monster.join_sharp(ground).join(stone, 0.1)
}


effect!(|Effect { uv, time, .. }, _, _| {
    let mut color = Vec3::ZERO;
    let sun_dir = Vec3::new(0.8, 0.55, 0.2).normalize();

    // Setup a camera and march a ray from it...
    let look_at = Vec3::new(-0.5, 0.9, 0.0);
    let ray = Ray::camera(uv, look_at, 5.0, 2.0, time * 0.1);

    match ray.march(|p| map(p, time))
    {
        Some(mat) =>
        {
            // Calculate the normals of "something"
            let pos = ray.shoot(mat.dist);
            let normal = calc_normal(pos, |p| map(p, time));

            // March shadow rays
            let shadow_ray = Ray::new(pos + (normal * HIT), sun_dir);
            let shadow_dist = shadow_ray.march(|p| map(p, time));

            // Calculate lighting
            let sun_light = f32::clamp(normal.dot(sun_dir), 0.0, 1.0);
            let sun_shadow = shadow_dist.map_or(1.0, |_| 0.0);
            let sky_light = f32::clamp(0.5 + 0.5 * normal.dot(Vec3::new(0.0, 1.0, 0.0)), 0.0, 8.0);
            let bounce_light =
                f32::clamp(0.5 + 0.5 * normal.dot(Vec3::new(0.0, -1.0, 0.0)), 0.0, 1.0);

            // Calculate materials
            let matt = match mat.mat
            {
                Mat::MonsterBody => Vec3::new(0.2, 0.01, 0.02),
                Mat::MonsterEye => Vec3::new(0.5, 0.5, 0.5),
                Mat::MonsterPupil => Vec3::splat(0.01),
                Mat::Ground => Vec3::new(0.05, 0.1, 0.02),
                Mat::Stone => Vec3::new(0.05, 0.04, 0.04),
            };

            // Apply lighting
            color += matt * Vec3::new(7.0, 5.0, 3.0) * sun_light * sun_shadow;
            color += matt * Vec3::new(0.5, 0.8, 0.9) * sky_light;
            color += matt * Vec3::new(0.7, 0.3, 0.2) * bounce_light;
        }

        None =>
        {
            // Draw a sky
            color = Vec3::new(0.2, 0.2, 1.0) - ray.dir.y.max(0.0) * 0.5;
            color = mix(color, Vec3::new(0.3, 0.3, 0.8), f32::exp(-10.0 * ray.dir.y));
        }
    }

    color.powf(0.5545)
});
