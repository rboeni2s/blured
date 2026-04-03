#![allow(unused)]
use crate::*;
use raymarch::*;

// Define Materials
material!(MatDist, Mat => MonsterBody, MonsterEye, Ground, Stone);


fn monster_sdf(pos: Vec3, time: f32) -> MatDist
{
    let t = (time * 0.8).fract();
    // let t = 0.5; // Freeze the animation

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

    let belly_sdf = MatDist::new(Mat::MonsterBody, ellipse_sdf(init_pos, Vec3::splat(0.25)));
    let head_sdf = MatDist::new(
        Mat::MonsterBody,
        ellipse_sdf(init_pos + Vec3::new(0.0, -0.28, 0.0), Vec3::splat(0.2)),
    );
    let head_back_sdf = MatDist::new(
        Mat::MonsterBody,
        ellipse_sdf(init_pos + Vec3::new(0.0, -0.28, 0.1), Vec3::splat(0.2)),
    );
    let eye_l = MatDist::new(
        Mat::MonsterEye,
        sphere_sdf(init_pos + Vec3::new(-0.1, -0.3, -0.14), 0.05),
    );
    let eye_r = MatDist::new(
        Mat::MonsterEye,
        sphere_sdf(init_pos + Vec3::new(0.1, -0.3, -0.14), 0.05),
    );

    let mut monster = belly_sdf.smin(head_sdf, 0.1);
    monster = monster.smin(head_back_sdf, 0.03);
    monster = monster.smin(eye_l, 0.002);
    monster = monster.smin(eye_r, 0.002);

    monster
}


fn map(pos: Vec3, time: f32) -> MatDist
{
    let monster = monster_sdf(pos, time);

    let stone = MatDist::new(
        Mat::Stone,
        ellipse_sdf(
            pos - Vec3::new(-1.0, -0.25, -1.0),
            Vec3::new(0.4, 0.2, 0.25),
        ),
    );

    let ground = MatDist::new(Mat::Ground, pos.y + 0.25);

    let mut scene = monster.min(ground);
    scene = scene.smin(stone, 0.1);

    scene
}


effect!(|Effect { uv, time, .. }, _, _| {
    let mut color = Vec3::ZERO;
    let sun_dir = Vec3::new(0.8, 0.55, 0.2).normalize();
    let matt = Vec3::splat(0.18);

    // Setup a camera and march a ray from it...
    let look_at = Vec3::new(-0.5, 0.9, 0.0);
    let ray = Ray::camera(uv, time, look_at, 5.0, 2.0);

    match ray.march(time, map)
    {
        Some(mat) =>
        {
            // Calculate the normals of "something"
            let pos = ray.shoot(mat.dist());
            let normal = calc_normal(pos, time, map);

            // March shadow rays
            let shadow_ray = Ray::new(pos + (normal * HIT), sun_dir);
            let shadow_dist = shadow_ray.march(time, map);

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

            match mat.mat()
            {
                Mat::MonsterBody => color *= Vec3::new(0.6, 0.1, 0.1),
                Mat::MonsterEye => color *= Vec3::new(0.8, 0.8, 0.8),
                Mat::Ground => color *= Vec3::new(0.1, 0.4, 0.1),
                Mat::Stone => color *= Vec3::new(0.1, 0.1, 0.1),
            }
        }

        None =>
        {
            // Draw a sky
            color = Vec3::new(0.2, 0.2, 1.0) - ray.dir.y.max(0.0) * 0.5;
            color = mix(color, Vec3::new(0.3, 0.3, 0.8), f32::exp(-10.0 * ray.dir.y));
        }
    }

    pow(color, 0.4545)
});
