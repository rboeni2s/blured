use crate::service::renderer::buffer::IndexBuffer;
use crate::service::renderer::buffer::SQUARE_INDICES;
use crate::service::renderer::buffer::SQUARE_VERTICES;
use crate::service::renderer::buffer::VertexBuffer;
use crate::service::renderer::buffer::create_bind_group;
use crate::service::renderer::camera::Camera;
use crate::service::renderer::camera::CameraBuffer;
use crate::service::renderer::image_scene::EffectParams;
use crate::service::renderer::image_scene::ImageScene;
use crate::service::renderer::texture::Texture;
use crate::service::renderer::texture::TextureBindGroupLayout;
use keep::Guard;


/// Statically linked spirv shader sources, used for effects written in rust!!
static BLURED_SHADER: &[u8] = include_bytes!(env!("BLURED_SHADER_PATH"));


pub struct ScenePipeline
{
    pub pipeline: wgpu::RenderPipeline,
    pub camera_bind_group: wgpu::BindGroup,
    pub texture_bind_group_layout: TextureBindGroupLayout,
    pub color_bind_group_layout: wgpu::BindGroupLayout,
    pub output_texture: Texture,
}


impl ScenePipeline
{
    pub fn new(device: &wgpu::Device, (width, height): (u32, u32)) -> Self
    {
        let camera = CameraBuffer::new(device, Camera::default(), width, height);
        let camera_bind_group = camera.create_bind_group(device);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::util::make_spirv(BLURED_SHADER),
        });

        let texture_bind_group_layout = TextureBindGroupLayout::new(device);

        let color_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });


        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[
                Some(&texture_bind_group_layout),
                Some(camera.layout()),
                Some(&color_bind_group_layout),
            ],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("scene::vertex"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[VertexBuffer::default_layout()],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("scene::fragment"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });

        let output_texture = Texture::new(
            device,
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        Self {
            pipeline,
            camera_bind_group,
            texture_bind_group_layout,
            color_bind_group_layout,
            output_texture,
        }
    }

    pub fn render_scene(&self, device: &wgpu::Device, queue: &wgpu::Queue, scene: &ImageScene)
    {
        // Create a encoder and begin a new render pass with it
        let mut encoder = device.create_command_encoder(&Default::default());

        let mut renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.output_texture.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::default()),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            ..Default::default()
        });

        renderpass.set_pipeline(&self.pipeline);
        renderpass.set_bind_group(1, &self.camera_bind_group, &[]);

        scene.draw_in_pass(device, &mut renderpass);

        // Submit and present
        drop(renderpass);
        queue.submit(Some(encoder.finish()));
    }
}

pub struct EffectPipeline
{
    // Shared pipelines
    pub blur_pipeline: Guard<wgpu::RenderPipeline>,

    pub pipeline_layout: wgpu::PipelineLayout,
    pub format: wgpu::TextureFormat,
    pub texture_bind_group_layout: TextureBindGroupLayout,
    pub vertex_buffer: VertexBuffer<'static>,
    pub index_buffer: IndexBuffer,
    pub effect_params_layout: wgpu::BindGroupLayout,
    pub strength_layout: wgpu::BindGroupLayout,
    pub neuro_pipeline: Guard<wgpu::RenderPipeline>,
    pub blank_pipeline: Guard<wgpu::RenderPipeline>,
    pub jumping_pipeline: Guard<wgpu::RenderPipeline>,
}

impl EffectPipeline
{
    pub fn create_pipeline(
        &self,
        device: &wgpu::Device,
        shader: &wgpu::ShaderModule,
        entry_point_name: &str,
    ) -> anyhow::Result<Guard<wgpu::RenderPipeline>>
    {
        let scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
        let pipeline = Self::create_pipeline_impl(
            device,
            &self.pipeline_layout,
            self.format,
            shader,
            entry_point_name,
        );

        if let Some(e) = pollster::block_on(scope.pop())
        {
            return Err(e.into());
        }

        Ok(pipeline)
    }

    fn create_pipeline_impl(
        device: &wgpu::Device,
        layout: &wgpu::PipelineLayout,
        format: wgpu::TextureFormat,
        shader: &wgpu::ShaderModule,
        entry_point_name: &str,
    ) -> Guard<wgpu::RenderPipeline>
    {
        device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(layout),
                vertex: wgpu::VertexState {
                    module: shader,
                    entry_point: Some("vertex"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    buffers: &[VertexBuffer::default_layout()],
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                fragment: Some(wgpu::FragmentState {
                    module: shader,
                    entry_point: Some(entry_point_name),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview_mask: None,
                cache: None,
            })
            .into()
    }

    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self
    {
        let texture_bind_group_layout = TextureBindGroupLayout::new(device);
        let effect_data_layout = EffectParams::layout(device);
        let strength_layout = EffectParams::layout(device);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[
                Some(&texture_bind_group_layout),
                Some(&effect_data_layout),
                Some(&strength_layout),
            ],
            immediate_size: 0,
        });

        let blur_shader =
            device.create_shader_module(wgpu::include_wgsl!("../../../wgsl_shader/blur.wgsl"));

        let neuro_shader =
            device.create_shader_module(wgpu::include_wgsl!("../../../wgsl_shader/neuro.wgsl"));

        // This module bundles all of blured shader-effects that are not written in wgsl!
        let blured_shader_bundle = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::util::make_spirv(BLURED_SHADER),
        });

        let blur_pipeline =
            Self::create_pipeline_impl(device, &pipeline_layout, format, &blur_shader, "fragment");

        let neuro_pipeline =
            Self::create_pipeline_impl(device, &pipeline_layout, format, &neuro_shader, "fragment");

        let blank_pipeline = Self::create_pipeline_impl(
            device,
            &pipeline_layout,
            format,
            &blured_shader_bundle,
            "blank::fragment",
        );

        let jumping_pipeline = Self::create_pipeline_impl(
            device,
            &pipeline_layout,
            format,
            &blured_shader_bundle,
            "jumping::fragment",
        );

        let vertex_buffer = VertexBuffer::new(device, SQUARE_VERTICES);
        let index_buffer = IndexBuffer::new(device, SQUARE_INDICES);

        Self {
            blur_pipeline,
            neuro_pipeline,
            blank_pipeline,
            jumping_pipeline,
            pipeline_layout,
            texture_bind_group_layout,
            vertex_buffer,
            index_buffer,
            effect_params_layout: effect_data_layout,
            strength_layout,
            format,
        }
    }

    #[allow(clippy::too_many_arguments)] // sometimes it just has to be...
    pub fn render_effect(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        output: &Texture,
        scene: &ImageScene,
        surface: &wgpu::TextureView,
        effect_strength: f32,
        elapsed_time: f32,
    )
    {
        // Create a encoder and begin a new render pass with it
        let mut encoder = device.create_command_encoder(&Default::default());

        let mut renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: surface,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::default()),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            ..Default::default()
        });

        let texture_bind_group = self
            .texture_bind_group_layout
            .create_bind_group(device, output);

        // vec2 with x: current strength, y: maximum strength
        let strength = [effect_strength, scene.effect_strength, elapsed_time];
        let strength = create_bind_group(device, &strength, &self.strength_layout);

        renderpass.set_pipeline(&scene.pipeline);
        renderpass.set_bind_group(0, &texture_bind_group, &[]);
        renderpass.set_bind_group(1, &scene.effect_params_bind_group, &[]);
        renderpass.set_bind_group(2, &strength, &[]);
        self.vertex_buffer.set_for_pass(&mut renderpass);
        self.index_buffer.set_for_pass(&mut renderpass);
        self.index_buffer.draw_index(&mut renderpass);

        // Submit and present
        drop(renderpass);
        queue.submit(Some(encoder.finish()));
    }
}
