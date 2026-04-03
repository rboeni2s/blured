use crate::*;
use core::ops::{AddAssign, SubAssign};

pub const HIT: f32 = 0.001;
pub const FAR: f32 = 20.0;


pub struct Ray
{
    pub ori: Vec3,
    pub dir: Vec3,
}


impl Ray
{
    pub fn new(ori: Vec3, dir: Vec3) -> Self
    {
        Self { ori, dir }
    }

    pub fn shoot(&self, t: f32) -> Vec3
    {
        self.ori + t * self.dir
    }

    pub fn march<T, F, M>(&self, time: f32, map: F) -> Option<T>
    where
        T: Material<M> + Default,
        F: Fn(Vec3, f32) -> T,
    {
        let mut t = T::default();

        loop
        {
            let pos = self.shoot(t.dist());
            let dist = map(pos, time);
            *t.mat_mut() = dist.mat();

            if dist.dist() < HIT
            {
                return Some(t);
            }
            else if t.dist() > FAR
            {
                return None;
            }

            t.dist_mut().add_assign(dist.dist());
        }
    }

    pub fn camera(uv: Vec2, time: f32, look_at: Vec3, zoom: f32, pitch: f32) -> Self
    {
        let pos = Vec2::new(zoom, pitch);
        let angle = time * 0.1;
        let origin = Vec3::new(angle.sin() * pos.x, pos.y, angle.cos() * pos.x);
        let ww = (look_at - origin).normalize();
        let uu = ww.cross(Vec3::new(0.0, 1.0, 0.0)).normalize();
        let vv = uu.cross(ww).normalize();
        let direction = (uv.x * uu + uv.y * vv + 1.5 * ww).normalize();

        Self::new(origin, direction)
    }
}


pub fn calc_normal<T, F, M>(pos: Vec3, time: f32, map: F) -> Vec3
where
    T: Material<M>,
    F: Fn(Vec3, f32) -> T,
{
    const NORMAL_ACC: Vec2 = Vec2::new(0.0001, 0.0);

    Vec3::new(
        map(pos + NORMAL_ACC.xyy(), time).dist() - map(pos - NORMAL_ACC.xyy(), time).dist(),
        map(pos + NORMAL_ACC.yxy(), time).dist() - map(pos - NORMAL_ACC.yxy(), time).dist(),
        map(pos + NORMAL_ACC.yyx(), time).dist() - map(pos - NORMAL_ACC.yyx(), time).dist(),
    )
    .normalize()
}


pub fn ellipse_sdf(pos: Vec3, radius: Vec3) -> f32
{
    let pr = pos / radius;
    let d0 = pr.length();
    let d1 = (pr / radius).length();
    d0 * (d0 - 1.0) / d1
}


pub fn sphere_sdf(pos: Vec3, radius: f32) -> f32
{
    pos.length() - radius
}


pub trait Material<M>
{
    fn mat_mut(&mut self) -> &mut M;
    fn mat(&self) -> M;
    fn dist(&self) -> f32;
    fn dist_mut(&mut self) -> &mut f32;
}


pub trait MaterialExt<M>: Material<M>
where
    M: PartialEq,
    Self: Sized,
{
    fn min(self, other: Self) -> Self
    {
        if self.dist() < other.dist()
        {
            self
        }
        else
        {
            other
        }
    }

    fn smin(self, other: Self, smooth: f32) -> Self
    {
        let h = f32::max(smooth - f32::abs(self.dist() - other.dist()), 0.0);
        let h = h * h / (smooth * 4.0);
        let mut min = self.min(other);
        min.dist_mut().sub_assign(h);
        min
    }

    fn is(self, other: M) -> bool
    {
        self.mat() == other
    }
}


impl<T, M> MaterialExt<M> for T
where
    T: Material<M>,
    M: PartialEq,
{
}


#[macro_export]
macro_rules! material {
    ($wname:ident, $mname:ident => $($mat:ident),+) => {
        #[repr(u32)]
        #[derive(Default, Copy, Clone, PartialEq, Eq)]
        enum $mname
        {
            #[default]
            $($mat),*
        }


        #[derive(Default, Copy, Clone, PartialEq)]
        struct $wname
        {
            material: $mname,
            dist: f32,
        }


        impl $wname
        {
            fn new(material: $mname, dist: f32) -> Self
            {
                Self { material, dist }
            }
        }


        impl Material<Mat> for $wname
        {
            fn mat(&self) -> $mname
            {
                self.material
            }

            fn mat_mut(&mut self) -> &mut $mname
            {
                &mut self.material
            }

            fn dist(&self) -> f32
            {
                self.dist
            }

            fn dist_mut(&mut self) -> &mut f32
            {
                &mut self.dist
            }
        }
    };
}
