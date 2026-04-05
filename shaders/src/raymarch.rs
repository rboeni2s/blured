use crate::*;

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

    pub fn march<F, M>(&self, map: F) -> Option<Sdf<M>>
    where
        M: Default,
        F: Fn(Vec3) -> Sdf<M>,
    {
        let mut t = Sdf::default();

        loop
        {
            let pos = self.shoot(t.dist);
            let dist = map(pos);
            t.mat = dist.mat;

            if dist.dist < HIT
            {
                return Some(t);
            }
            else if t.dist > FAR
            {
                return None;
            }

            t.dist += dist.dist;
        }
    }

    pub fn camera(uv: Vec2, look_at: Vec3, zoom: f32, pitch: f32, angle: f32) -> Self
    {
        let pos = Vec2::new(zoom, pitch);
        let origin = Vec3::new(angle.sin() * pos.x, pos.y, angle.cos() * pos.x);
        let ww = (look_at - origin).normalize();
        let uu = ww.cross(Vec3::new(0.0, 1.0, 0.0)).normalize();
        let vv = uu.cross(ww).normalize();
        let direction = (uv.x * uu + uv.y * vv + 1.5 * ww).normalize();

        Self::new(origin, direction)
    }
}


pub fn calc_normal<F, M>(pos: Vec3, map: F) -> Vec3
where
    F: Fn(Vec3) -> Sdf<M>,
{
    const NORMAL_ACC: Vec2 = Vec2::new(0.0001, 0.0);

    Vec3::new(
        map(pos + NORMAL_ACC.xyy()).dist - map(pos - NORMAL_ACC.xyy()).dist,
        map(pos + NORMAL_ACC.yxy()).dist - map(pos - NORMAL_ACC.yxy()).dist,
        map(pos + NORMAL_ACC.yyx()).dist - map(pos - NORMAL_ACC.yyx()).dist,
    )
    .normalize()
}


/// Applies gamma correction and dithering
pub fn finalize(uv: Vec2, col: Vec3) -> Vec3
{
    col.powf(0.45) + ((uv.x * 114.0 * 11.0).sin() * (uv.y * 211.1 * 11.0).sin() / 600.0)
}


/// Multisamples `f` to antialiase the image
pub fn antialiase(aa: u32, uv: Vec2, f: impl Fn(Vec2) -> Vec3) -> Vec3
{
    let mut color = Vec3::ZERO;

    for off_x in 0..aa
    {
        for off_y in 0..aa
        {
            let offset = (Vec2::new(off_x as f32, off_y as f32) / aa as f32 - 0.5) * HIT;
            color += f(uv + offset);
        }
    }

    color / (aa * aa) as f32
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


pub fn plane_sdf(pos: Vec3) -> f32
{
    pos.y
}


#[macro_export]
macro_rules! material {
    ($mname:ident => [$($mat:ident),+]) => {
        #[repr(u32)]
        #[derive(Default, Copy, Clone, PartialEq, Eq)]
        enum $mname
        {
            #[default]
            $($mat),*
        }
    };
}


pub struct Sdf<M>
{
    pub dist: f32,
    pub mat: M,
}


impl<M> Sdf<M>
where
    M: PartialEq,
{
    pub fn join_sharp(self, other: impl Into<Self>) -> Self
    {
        let other = other.into();
        if self.dist < other.dist { self } else { other }
    }

    pub fn join(self, other: impl Into<Self>, smooth: f32) -> Self
    {
        let other = other.into();
        let h = f32::max(smooth - f32::abs(self.dist - other.dist), 0.0);
        let h = h * h / (smooth * 4.0);
        let mut min = self.join_sharp(other);
        min.dist -= h;
        min
    }

    pub fn carve_sharp(mut self, other: impl Into<Self>) -> Self
    {
        let other = other.into();
        self.dist = self.dist.max(-other.dist);
        self
    }

    pub fn carve(self, other: impl Into<Self>, smooth: f32) -> Self
    {
        let other = other.into();
        let h = f32::max(smooth - f32::abs(self.dist + other.dist), 0.0);
        let h = h * h / (smooth * 4.0);
        let mut max = self.carve_sharp(other);
        max.dist += h;
        max
    }

    pub fn is(self, other: M) -> bool
    {
        self.mat == other
    }
}


impl<M> AsRef<M> for Sdf<M>
{
    fn as_ref(&self) -> &M
    {
        &self.mat
    }
}


impl<M> AsMut<M> for Sdf<M>
{
    fn as_mut(&mut self) -> &mut M
    {
        &mut self.mat
    }
}


impl<M> Default for Sdf<M>
where
    M: Default,
{
    fn default() -> Self
    {
        Self {
            dist: 0.0,
            mat: Default::default(),
        }
    }
}


impl<F, M> From<SdfBuilder<F, M>> for Sdf<M>
where
    F: Fn(Vec3) -> f32,
    M: Default + Copy + Clone + PartialEq + Eq,
{
    fn from(value: SdfBuilder<F, M>) -> Self
    {
        value.build()
    }
}


pub struct SdfBuilder<F, M>
{
    pos: Vec3,
    func: F,
    mat: M,
    ray_tip: Vec3,
}


impl<M, F> SdfBuilder<F, M>
where
    F: Fn(Vec3) -> f32,
    M: Default + Copy + Clone + PartialEq + Eq,
{
    #[inline]
    pub fn new(ray_tip: Vec3, func: F) -> Self
    {
        Self {
            ray_tip,
            func,
            mat: M::default(),
            pos: ray_tip,
        }
    }

    #[inline]
    pub fn pos(self, x: f32, y: f32, z: f32) -> Self
    {
        self.posv(Vec3::new(x, y, z))
    }

    #[inline]
    pub fn posv(mut self, pos: Vec3) -> Self
    {
        self.pos = self.ray_tip - pos;
        self
    }

    #[inline]
    pub fn rot_x(mut self, angle: f32) -> Self
    {
        self.pos = self.pos.rotate_x(angle.to_radians());
        self
    }

    #[inline]
    pub fn rot_y(mut self, angle: f32) -> Self
    {
        self.pos = self.pos.rotate_y(angle.to_radians());
        self
    }

    #[inline]
    pub fn rot_z(mut self, angle: f32) -> Self
    {
        self.pos = self.pos.rotate_z(angle.to_radians());
        self
    }


    #[inline]
    pub fn mat(mut self, mat: M) -> Self
    {
        self.mat = mat;
        self
    }

    #[inline]
    pub fn build(self) -> Sdf<M>
    {
        // F needs to have a local binding because of a
        // pointer offset error in the spirv codegen backend...
        let f = self.func;

        Sdf {
            dist: f(self.pos),
            mat: self.mat,
        }
    }
}


pub fn sdf<F, M>(ray_tip: Vec3, func: F) -> SdfBuilder<F, M>
where
    F: Fn(Vec3) -> f32,
    M: Default + Copy + Clone + PartialEq + Eq,
{
    SdfBuilder::new(ray_tip, func)
}
