use crate::service::renderer::{
    buffer::{IndexBuffer, Vertex, VertexBuffer, create_bind_group},
    pipelines::{EffectPipeline, ScenePipeline},
    texture::{AsocTexture, Image},
};
use keep::Guard;
use wgpu::BindGroupLayoutEntry;


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


#[allow(unused)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect
{
    Blur,
    Neuro,
    Custom(String),
}


impl Effect
{
    pub fn fetch_pipeline(
        &self,
        device: &wgpu::Device,
        pipeline: &EffectPipeline,
    ) -> anyhow::Result<Guard<wgpu::RenderPipeline>>
    {
        Ok(match self
        {
            // Get guards to the shared builtin pipelines
            Effect::Blur => pipeline.blur_pipeline.clone(),
            Effect::Neuro => pipeline.neuro_pipeline.clone(),

            // Load a user supplied wgsl shader from disk
            Effect::Custom(path) =>
            {
                let data = std::fs::read_to_string(path)?;
                let scope_guard = device.push_error_scope(wgpu::ErrorFilter::Validation);

                let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("User Shader Module"),
                    source: wgpu::ShaderSource::Wgsl(data.into()),
                });

                if let Some(error) = pollster::block_on(scope_guard.pop())
                {
                    return Err(error.into());
                }

                pipeline.create_pipeline(device, &shader)?
            }
        })
    }
}


#[repr(C)]
#[derive(Clone)]
pub struct EffectParams
{
    pub param_a: [f32; 4],
    pub param_b: [f32; 4],
    pub param_c: [f32; 4],
}


impl EffectParams
{
    pub fn layout(device: &wgpu::Device) -> wgpu::BindGroupLayout
    {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        })
    }
}


impl Default for EffectParams
{
    fn default() -> Self
    {
        Self {
            param_a: [1.0, 0.0, 0.0, 1.0],
            param_b: [0.0, 1.0, 0.0, 1.0],
            param_c: [0.0, 0.0, 1.0, 1.0],
        }
    }
}


pub struct ImageScene
{
    pub ident: String,
    pub vertex_buffer: VertexBuffer<'static>,
    pub index_buffer: IndexBuffer,
    pub texture_bind_group: wgpu::BindGroup,
    pub background_bind_group: wgpu::BindGroup,
    pub effect_params_bind_group: wgpu::BindGroup,
    pub effect_strength: f32,
    pub dynamic: bool,
    pub pipeline: Guard<wgpu::RenderPipeline>,
}


impl ImageScene
{
    pub fn new(
        desc: &ImageSceneDesc,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene_pipeline: &ScenePipeline,
        effect_pipeline: &EffectPipeline,
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

        let background_bind_group = create_bind_group(
            device,
            &desc.background,
            &scene_pipeline.color_bind_group_layout,
        );
        let effect_params_bind_group = create_bind_group(
            device,
            &desc.effect_params,
            &effect_pipeline.effect_params_layout,
        );

        let texture = AsocTexture::from_image(device, queue, image);

        // Force dynamic rendering on certain effects.
        let dynamic = desc.dynamic || desc.effect == Effect::Neuro;

        Ok(Self {
            ident: desc.ident.clone(),
            vertex_buffer: VertexBuffer::new(device, verts.as_flattened()),
            index_buffer: IndexBuffer::new(device, &INDICES),
            texture_bind_group: scene_pipeline
                .texture_bind_group_layout
                .create_bind_group(device, texture.texture()),
            background_bind_group,
            effect_params_bind_group,
            dynamic,
            effect_strength: desc.effect_strength,
            pipeline: desc.effect.fetch_pipeline(device, effect_pipeline)?,
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
    pub effect_params: EffectParams,
    pub effect_strength: f32,
    pub effect: Effect,
}


impl ImageSceneDesc
{
    pub fn load(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene_pipeline: &ScenePipeline,
        effect_pipeline: &EffectPipeline,
        surface_size: (u32, u32),
    ) -> anyhow::Result<ImageScene>
    {
        ImageScene::new(
            self,
            device,
            queue,
            scene_pipeline,
            effect_pipeline,
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
            effect_params: EffectParams::default(),
            effect_strength: 50.0,
            effect: Effect::Blur,
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
