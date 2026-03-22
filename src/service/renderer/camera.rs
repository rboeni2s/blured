#![allow(unused)]


use cgmath::{Matrix4, Point3, Vector3};
use keep::Keep;
use wgpu::util::DeviceExt;


pub struct Camera
{
    origin: Keep<Point3<f32>>,
    target: Keep<Point3<f32>>,
    up: Vector3<f32>,
    aspect: Keep<f32>,
    fov: f32,
    near: f32,
    far: f32,
    proj_buffer: Keep<Option<wgpu::Buffer>>,
}


impl Camera
{
    pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::from_cols(
        cgmath::Vector4::new(1.0, 0.0, 0.0, 0.0),
        cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0),
        cgmath::Vector4::new(0.0, 0.0, 0.5, 0.0),
        cgmath::Vector4::new(0.0, 0.0, 0.5, 1.0),
    );

    pub fn build_view_projection_matrix(&self, width: f32, height: f32) -> Matrix4<f32>
    {
        let view = Matrix4::look_at_rh(*self.origin.read(), *self.target.read(), self.up);
        // let proj = cgmath::perspective(Deg(self.fov), *self.aspect.read(), self.near, self.far);
        let proj = cgmath::ortho(
            -width * 0.5,
            width * 0.5,
            -height * 0.5,
            height * 0.5,
            self.near,
            self.far,
        );
        Self::OPENGL_TO_WGPU_MATRIX * proj * view
    }

    pub fn update_aspect(&self, device: &wgpu::Device, width: u32, height: u32)
    {
        let aspect = width as f32 / height as f32;
        self.aspect.write(aspect);
        self.rebuild_projection_matrix(device, width, height);
    }

    pub fn rebuild_projection_matrix(&self, device: &wgpu::Device, width: u32, height: u32)
    {
        // Get the view projection matrix
        let mat: [[f32; 4]; 4] = self
            .build_view_projection_matrix(width as f32, height as f32)
            .into();

        // Cast it into a &[u8] buffer
        let mat_size = size_of::<[[f32; 4]; 4]>();
        let mat_ptr = &mat as *const _ as *const u8;
        let mat = unsafe { std::slice::from_raw_parts(mat_ptr, mat_size) };

        // Create the buffer
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Projection Buffer"),
            contents: mat,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        self.proj_buffer.write(Some(buffer));
    }

    #[inline]
    pub fn set_origin(&self, origin: Point3<f32>) -> Point3<f32>
    {
        *self.origin.swap(origin)
    }
}


impl Default for Camera
{
    fn default() -> Self
    {
        Self {
            origin: Point3::new(0.0, 0.0, 0.1).into(),
            target: Point3::new(0.0, 0.0, 0.0).into(),
            up: Vector3::unit_y(),
            aspect: Keep::new(1.0),
            fov: 50.0,
            near: 0.1,
            far: 100.0,
            proj_buffer: Keep::new(None),
        }
    }
}


pub struct CameraBuffer
{
    layout: wgpu::BindGroupLayout,
    camera: Camera,
}


impl CameraBuffer
{
    pub fn new(device: &wgpu::Device, camera: Camera, width: u32, height: u32) -> Self
    {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Camera Projection Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        // Ensure camera projection matrix exists
        camera.rebuild_projection_matrix(device, width, height);

        Self { layout, camera }
    }

    pub fn layout(&self) -> &wgpu::BindGroupLayout
    {
        &self.layout
    }

    pub fn create_bind_group(&self, device: &wgpu::Device) -> wgpu::BindGroup
    {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Projection Bind Group"),
            layout: &self.layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: self
                    .camera
                    .proj_buffer
                    .read()
                    .as_ref()
                    .as_ref()
                    .unwrap() // The buffer will exist because it is rebuilt in Self::new
                    .as_entire_binding(),
            }],
        })
    }
}


impl std::ops::Deref for CameraBuffer
{
    type Target = Camera;

    fn deref(&self) -> &Self::Target
    {
        &self.camera
    }
}
