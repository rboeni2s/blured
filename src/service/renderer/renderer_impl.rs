use crate::service::{
    renderer::image_scene::{ImageScene, ImageSceneDesc},
    renderer::pipelines::{EffectPipeline, ScenePipeline},
    wlclient::WindowHandle,
};
use anyhow::Context;
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    nonpoison::Mutex,
};


#[allow(unused)]
pub struct RendererImpl
{
    instance: wgpu::Instance,
    surface: Mutex<Option<wgpu::Surface<'static>>>,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    width: u32,
    height: u32,
    default_scene: ImageScene,
    scene_index: AtomicUsize,
    scene_out_of_date: AtomicBool,
    scenes: Vec<ImageScene>,
    scene_pipeline: ScenePipeline,
    effect_pipeline: EffectPipeline,
}


impl RendererImpl
{
    pub fn new(window_handle: WindowHandle, scene_desc: &[ImageSceneDesc]) -> anyhow::Result<Self>
    {
        let (width, height) = window_handle.surface_size;

        let instance =
            wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle_from_env());

        let surface = unsafe {
            instance.create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
                raw_display_handle: Some(window_handle.display_handle),
                raw_window_handle: window_handle.window_handle,
            })
        }?;

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
        }))?;

        let (device, queue) = pollster::block_on(adapter.request_device(&Default::default()))?;

        let cap = surface.get_capabilities(&adapter);
        let format = *cap
            .formats
            .first()
            .context("Surface has no supported formats ???")?;

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width,
            height,
            present_mode: wgpu::PresentMode::Mailbox,
            desired_maximum_frame_latency: 2,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![format],
        };

        surface.configure(&device, &surface_config);

        let effect_pipeline = EffectPipeline::new(&device, format);
        let scene_pipeline = ScenePipeline::new(&device, window_handle.surface_size);

        let mut scenes = Vec::with_capacity(scene_desc.len());
        for desc in scene_desc
        {
            scenes.push(desc.load(
                &device,
                &queue,
                &scene_pipeline.color_bind_group_layout,
                &scene_pipeline.texture_bind_group_layout,
                window_handle.surface_size,
            )?);
        }

        let default_scene = ImageSceneDesc::default().load(
            &device,
            &queue,
            &scene_pipeline.color_bind_group_layout,
            &scene_pipeline.texture_bind_group_layout,
            window_handle.surface_size,
        )?;

        Ok(Self {
            instance,
            surface: Mutex::new(Some(surface)),
            adapter,
            device,
            queue,
            width,
            height,
            scenes,
            default_scene,
            scene_index: AtomicUsize::new(0),
            scene_pipeline,
            effect_pipeline,
            scene_out_of_date: AtomicBool::new(true),
        })
    }

    pub fn render(&self) -> anyhow::Result<()>
    {
        // Get the surface texture
        let surface: &Option<wgpu::Surface> = &self.surface.lock();
        let surface_texture = match surface
            .as_ref()
            .context("No surface")?
            .get_current_texture()
        {
            wgpu::CurrentSurfaceTexture::Success(texture)
            | wgpu::CurrentSurfaceTexture::Suboptimal(texture) => texture,
            e =>
            {
                return Err(anyhow::Error::msg(format!(
                    "Failed to fetch current surface texture due to: {e:?}"
                )));
            }
        };

        // Create a texture view from the surface texture
        let texture_view = surface_texture.texture.create_view(&Default::default());

        // Only rerender the scene if it is out of date
        if self.scene_out_of_date.load(Ordering::Acquire)
        {
            let scene = self
                .scenes
                .get(self.scene_index.load(Ordering::Relaxed))
                .unwrap_or(&self.default_scene);

            self.scene_pipeline
                .render_scene(&self.device, &self.queue, scene);

            let _ = self.scene_out_of_date.compare_exchange(
                true,
                false,
                Ordering::Release,
                Ordering::Relaxed,
            );
        }

        self.effect_pipeline.render_effect(
            &self.device,
            &self.queue,
            &self.scene_pipeline.output_texture,
            &texture_view,
        );

        surface_texture.present();
        Ok(())
    }

    /// Destroys the wgpu surface, nothing can be rendered without reinitializing the renderer after calling this function.
    ///
    /// # Warning
    /// This function must be called before the wayland surface is destroyed!!!
    pub fn destroy_surface(&self)
    {
        if let Some(surface) = self.surface.lock().take()
        {
            drop(surface);
        }
    }

    /// Tries to find a scene with the given identifier and switches to it, returning the scene index.
    pub fn switch_scene(&self, ident: &str) -> anyhow::Result<usize>
    {
        let index = self
            .scenes
            .iter()
            .position(|e| e.ident == ident)
            .context(format!("No scene named: {ident:?}"))?;

        self.scene_index.store(index, Ordering::Relaxed);
        self.scene_out_of_date.store(true, Ordering::Relaxed);

        Ok(index)
    }
}
