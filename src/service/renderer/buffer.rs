#![allow(unused)]


use wgpu::util::DeviceExt;


pub const SQUARE_INDICES: &[u16] = &[0, 1, 2, 3, 2, 1];
pub const SQUARE_VERTICES: &[Vertex] = &[
    Vertex {
        pos: [-1., -1., 0.0],
        tex: [0.0, 0.0],
    },
    Vertex {
        pos: [1.0, -1., 0.0],
        tex: [1.0, 0.0],
    },
    Vertex {
        pos: [-1., 1.0, 0.0],
        tex: [0.0, 1.0],
    },
    Vertex {
        pos: [1.0, 1.0, 0.0],
        tex: [1.0, 1.0],
    },
];


#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Vertex
{
    pub pos: [f32; 3],
    pub tex: [f32; 2],
}


impl Vertex
{
    pub const VERTEX_ATTRIBUTES: &'static [wgpu::VertexAttribute] = &[
        // The position vector is first with 3xf32
        wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Float32x3,
            offset: 0,
            shader_location: 0,
        },
        // After that is the texture coordinate (tex) vector with 2xf32
        wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Float32x2,
            offset: size_of::<[f32; 3]>() as u64, // The tex vector has to be offset by the size of the position vector
            shader_location: 1,
        },
    ];
}


pub struct VertexBuffer<'a>
{
    length: u32,
    buffer: wgpu::Buffer,
    layout: wgpu::VertexBufferLayout<'a>,
}


impl<'a> VertexBuffer<'a>
{
    pub fn new(device: &wgpu::Device, buffer: &[Vertex]) -> Self
    {
        let length = buffer.len() as u32;

        // Cast slice of vertices into as slice of bytes
        let buffer_size = size_of_val(buffer);
        let buffer_bytes = buffer.as_ptr() as *const u8;
        let buffer = unsafe { std::slice::from_raw_parts(buffer_bytes, buffer_size) };

        Self::new_with(device, buffer, length, Vertex::VERTEX_ATTRIBUTES)
    }

    pub fn new_with(
        device: &wgpu::Device,
        buffer: &[u8],
        length: u32,
        attributes: &'a [wgpu::VertexAttribute],
    ) -> Self
    {
        // Create the vertex buffer
        let buffer_descriptor = wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: buffer,
            usage: wgpu::BufferUsages::VERTEX,
        };

        let buffer = device.create_buffer_init(&buffer_descriptor);

        // Define the layout
        let layout = wgpu::VertexBufferLayout {
            array_stride: size_of::<Vertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes,
        };

        VertexBuffer {
            buffer,
            layout,
            length,
        }
    }

    pub fn layout(&self) -> &wgpu::VertexBufferLayout<'a>
    {
        &self.layout
    }

    pub fn length(&self) -> u32
    {
        self.length
    }

    pub fn set_for_pass(&self, render_pass: &mut wgpu::RenderPass)
    {
        render_pass.set_vertex_buffer(0, self.buffer.slice(..));
    }

    pub fn default_layout() -> wgpu::VertexBufferLayout<'static>
    {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Vertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: Vertex::VERTEX_ATTRIBUTES,
        }
    }
}


pub struct IndexBuffer
{
    length: u32,
    buffer: wgpu::Buffer,
    format: wgpu::IndexFormat,
}


impl IndexBuffer
{
    pub fn new(device: &wgpu::Device, buffer: &[u16]) -> Self
    {
        let length = buffer.len() as u32;

        // Cast slice of vertices into as slice of bytes
        let buffer_size = size_of_val(buffer);
        let buffer_bytes = buffer.as_ptr() as *const u8;
        let buffer = unsafe { std::slice::from_raw_parts(buffer_bytes, buffer_size) };

        Self::new_with(device, buffer, length, wgpu::IndexFormat::Uint16)
    }

    pub fn new_with(
        device: &wgpu::Device,
        buffer: &[u8],
        length: u32,
        format: wgpu::IndexFormat,
    ) -> Self
    {
        let buffer_descriptor = wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: buffer,
            usage: wgpu::BufferUsages::INDEX,
        };

        let buffer = device.create_buffer_init(&buffer_descriptor);

        Self {
            length,
            buffer,
            format,
        }
    }

    pub fn format(&self) -> wgpu::IndexFormat
    {
        self.format
    }

    pub fn length(&self) -> u32
    {
        self.length
    }

    pub fn set_for_pass(&self, render_pass: &mut wgpu::RenderPass)
    {
        render_pass.set_index_buffer(self.buffer.slice(..), self.format);
    }

    pub fn draw_index(&self, render_pass: &mut wgpu::RenderPass)
    {
        render_pass.draw_indexed(0..self.length, 0, 0..1);
    }
}


pub fn create_bind_group<T>(
    device: &wgpu::Device,
    data: &T,
    layout: &wgpu::BindGroupLayout,
) -> wgpu::BindGroup
{
    // Cast &T into &[u8]
    let buf_size = size_of_val(data);
    let buf_ptr = data as *const _ as *const u8;
    let buf = unsafe { std::slice::from_raw_parts(buf_ptr, buf_size) };

    // Create a buffer
    let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: buf,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    // Put the buffer in a bind group
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: buffer.as_entire_binding(),
        }],
    })
}
