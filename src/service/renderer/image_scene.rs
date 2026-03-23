use wgpu::util::DeviceExt;

use crate::service::renderer::{
    buffer::{IndexBuffer, Vertex, VertexBuffer},
    texture::{AsocTexture, Image, TextureBindGroupLayout},
};


#[allow(unused)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub enum ImageFit
{
    Stretch,
    FillH,
    #[default]
    FillV,
    Original,
}


pub struct ImageScene
{
    pub ident: String,
    pub vertex_buffer: VertexBuffer<'static>,
    pub index_buffer: IndexBuffer,
    pub texture_bind_group: wgpu::BindGroup,
    pub background_bind_group: wgpu::BindGroup,
    pub dynamic: bool,
}


impl ImageScene
{
    pub fn new(
        desc: &ImageSceneDesc,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        background_layout: &wgpu::BindGroupLayout,
        texture_layout: &TextureBindGroupLayout,
        (surface_width, surface_height): (u32, u32),
    ) -> anyhow::Result<Self>
    {
        const INDICES: [u16; 12] = [4, 5, 6, 7, 6, 5, 0, 1, 2, 3, 2, 1];

        let image = Image::new(&desc.image_source)?;
        let (surface_width, surface_height) =
            (surface_width as f32 * 0.5, surface_height as f32 * 0.5);
        let (image_width, image_height) = (image.width() as f32 * 0.5, image.height() as f32 * 0.5);
        let x_ratio = surface_width / image_width;
        let y_ratio = surface_height / image_height;

        let image_verts = match desc.image_fit
        {
            ImageFit::Stretch => make_plane(surface_width, surface_height),
            ImageFit::FillH => make_plane(surface_width, image_height * x_ratio),
            ImageFit::FillV => make_plane(image_width * y_ratio, surface_height),
            ImageFit::Original => make_plane(image_width, image_height),
        };

        let background_verts = make_background(surface_width, surface_height);
        let verts = [image_verts, background_verts];

        // Create the background color buffer
        let buf_size = size_of_val(&desc.background);
        let buf_ptr = &desc.background as *const _ as *const u8;
        let buf = unsafe { std::slice::from_raw_parts(buf_ptr, buf_size) };

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Projection Buffer"),
            contents: buf,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Put the buffer into the bind group
        let color_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: background_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        let texture = AsocTexture::from_image(device, queue, image);

        Ok(Self {
            ident: desc.ident.clone(),
            vertex_buffer: VertexBuffer::new(device, verts.as_flattened()),
            index_buffer: IndexBuffer::new(device, &INDICES),
            texture_bind_group: texture_layout.create_bind_group(device, texture.texture()),
            background_bind_group: color_bind_group,
            dynamic: desc.dynamic,
        })
    }

    pub fn draw_in_pass(&self, _device: &wgpu::Device, pass: &mut wgpu::RenderPass)
    {
        pass.set_bind_group(0, &self.texture_bind_group, &[]);
        pass.set_bind_group(2, &self.background_bind_group, &[]);
        self.index_buffer.set_for_pass(pass);
        self.vertex_buffer.set_for_pass(pass);
        self.index_buffer.draw_index(pass);
    }
}


pub struct ImageSceneDesc
{
    pub ident: String,
    pub image_source: Vec<u8>,
    pub image_fit: ImageFit,
    pub background: [f32; 3],
    pub dynamic: bool,
}


impl ImageSceneDesc
{
    pub fn load(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        background_layout: &wgpu::BindGroupLayout,
        texture_layout: &TextureBindGroupLayout,
        surface_size: (u32, u32),
    ) -> anyhow::Result<ImageScene>
    {
        ImageScene::new(
            self,
            device,
            queue,
            background_layout,
            texture_layout,
            surface_size,
        )
    }
}


impl Default for ImageSceneDesc
{
    fn default() -> Self
    {
        Self {
            ident: "default".into(),
            image_source: include_bytes!("../../../textures/astro_miku.jpg").to_vec(),
            image_fit: Default::default(),
            background: [0.055 * 0.5, 0.12 * 0.5, 0.2 * 0.5],
            dynamic: false,
        }
    }
}


fn make_plane(x: f32, y: f32) -> [Vertex; 4]
{
    [
        Vertex {
            pos: [-x, -y, 0.0],
            tex: [0.0, 0.0],
        },
        Vertex {
            pos: [x, -y, 0.0],
            tex: [1.0, 0.0],
        },
        Vertex {
            pos: [-x, y, 0.0],
            tex: [0.0, 1.0],
        },
        Vertex {
            pos: [x, y, 0.0],
            tex: [1.0, 1.0],
        },
    ]
}


fn make_background(x: f32, y: f32) -> [Vertex; 4]
{
    [
        Vertex {
            pos: [-x, -y, -1.0],
            tex: [-1.0, -1.0],
        },
        Vertex {
            pos: [x, -y, -1.0],
            tex: [-1.0, -1.0],
        },
        Vertex {
            pos: [-x, y, -1.0],
            tex: [-1.0, -1.0],
        },
        Vertex {
            pos: [x, y, -1.0],
            tex: [-1.0, -1.0],
        },
    ]
}
