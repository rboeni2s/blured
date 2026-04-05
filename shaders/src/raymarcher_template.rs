use crate::*;
use raymarch::*;


type Sdf = crate::raymarch::Sdf<Mat>;


material!(Mat => [Normal]);
effect!(|Effect { uv, time, .. }, _, _| raymarcher(uv, time));


fn map(pos: Vec3, _time: f32) -> Sdf
{
    sdf(pos, |p| ellipse_sdf(p, Vec3::new(0.2, 0.2, 0.5))).build()
}


fn raymarcher(uv: Vec2, time: f32) -> Vec3
{
    let color = antialiase(4, uv, |uv| {
        let camera = Ray::camera(uv, Vec3::ZERO, 3.0, 0.4, time * 0.3);

        match camera.march(|p| map(p, time))
        {
            Some(obj) => shade(obj, camera, time),
            None => Vec3::splat(1.0 + 1.8 * uv.y) * 0.0001,
        }
    });

    finalize(uv, color)
}


fn shade(obj: Sdf, cam: Ray, time: f32) -> Vec3
{
    let hit_pos = cam.shoot(obj.dist);
    let normals = calc_normal(hit_pos, |p| map(p, time));

    let color = match obj.mat
    {
        Mat::Normal => 0.5 + 0.5 * normals,
    };

    color
}
