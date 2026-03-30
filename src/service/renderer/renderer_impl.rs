use std::time::Duration;

use crate::{
    scene_desc::ImageSceneDesc,
    service::{
        renderer::{
            RenderResult,
            image_scene::ImageScene,
            pipelines::{EffectPipeline, ScenePipeline},
        },
        wlclient::WindowHandle,
    },
};
use anyhow::Context;


//TODO: Make this configurable...
const EFFECT_TRANSITION_SECS: f32 = 0.2;


#[allow(unused)]
pub struct RendererImpl
{
    instance: wgpu::Instance,
    surface: Option<wgpu::Surface<'static>>,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    width: u32,
    height: u32,
    default_scene: ImageScene,
    scene_index: usize,
    scene_out_of_date: bool,
    scenes: Vec<ImageScene>,
    scene_pipeline: ScenePipeline,
    effect_pipeline: EffectPipeline,
    effect_strength: f32,
    effect_on: bool,
    scene_effect_strength: f32,
    effect_change: f32,
    elapsed_time: f32,
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
                &scene_pipeline,
                &effect_pipeline,
                window_handle.surface_size,
            )?);
        }

        let default_scene = ImageSceneDesc::default().load(
            &device,
            &queue,
            &scene_pipeline,
            &effect_pipeline,
            window_handle.surface_size,
        )?;

        let scene_effect_strength = scenes.first().unwrap_or(&default_scene).effect_strength;

        Ok(Self {
            instance,
            surface: Some(surface),
            adapter,
            device,
            queue,
            width,
            height,
            scenes,
            default_scene,
            scene_index: 0,
            scene_pipeline,
            effect_pipeline,
            scene_out_of_date: true,
            effect_strength: 0.0,
            scene_effect_strength,
            effect_on: false,
            effect_change: (scene_effect_strength / EFFECT_TRANSITION_SECS).abs(),
            elapsed_time: 0.0,
        })
    }

    pub fn render(&mut self, delta: Duration) -> anyhow::Result<RenderResult>
    {
        self.elapsed_time += delta.as_secs_f32();
        let mut render_result = self.adjust_effect_strength(delta);

        // Get the surface texture
        let surface_texture = match (self.surface)
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

        // Try to get the current scene or fallback to the default scene
        let scene = self
            .scenes
            .get(self.scene_index)
            .unwrap_or(&self.default_scene);

        // Only rerender the scene if it is out of date
        if self.scene_out_of_date
        {
            self.scene_pipeline
                .render_scene(&self.device, &self.queue, scene);

            self.scene_out_of_date = false;
        }

        self.effect_pipeline.render_effect(
            &self.device,
            &self.queue,
            &self.scene_pipeline.output_texture,
            scene,
            &texture_view,
            self.effect_strength,
            self.elapsed_time,
        );

        // If the scene is dynamic, render again on the next frame
        if scene.dynamic
        {
            render_result = RenderResult::OutOfDate;
        }

        surface_texture.present();
        Ok(render_result)
    }

    /// Destroys the wgpu surface, nothing can be rendered without reinitializing the renderer after calling this function.
    ///
    /// # Warning
    /// This function must be called before the wayland surface is destroyed!!!
    pub fn destroy_surface(&mut self)
    {
        if let Some(surface) = self.surface.take()
        {
            drop(surface);
        }
    }

    /// Tries to find a scene with the given identifier and switches to it, returning the scene index.
    pub fn switch_scene(&mut self, ident: &str) -> anyhow::Result<usize>
    {
        let index = self
            .scenes
            .iter()
            .position(|e| e.ident == ident)
            .context(format!("No scene named: {ident:?}"))?;

        self.scene_effect_strength = self
            .scenes
            .first()
            .unwrap_or(&self.default_scene)
            .effect_strength;

        self.scene_index = index;
        self.effect_change = (self.scene_effect_strength / EFFECT_TRANSITION_SECS).abs();
        self.adjust_effect_strength(Duration::from_millis(0));
        self.scene_out_of_date = true;
        self.elapsed_time = 0.0;

        Ok(index)
    }

    fn adjust_effect_strength(&mut self, delta: Duration) -> RenderResult
    {
        if (self.effect_on && self.effect_strength == self.scene_effect_strength)
            || (!self.effect_on && self.effect_strength == 0.0)
        {
            return RenderResult::Clean;
        }

        let is_positive = self.scene_effect_strength >= 0.0;
        let scene_effect_strength = self.scene_effect_strength.abs();
        let effect_strength = self.effect_strength.abs();

        let mut strength;

        // go towards scene_effect_strength
        if self.effect_on
        {
            strength = effect_strength + (delta.as_secs_f32() * self.effect_change);
        }
        // go towards 0
        else
        {
            strength = effect_strength - (delta.as_secs_f32() * self.effect_change);
        }

        strength = strength.clamp(0.0, scene_effect_strength);

        if !is_positive
        {
            strength = -strength;
        }

        self.effect_strength = strength;

        RenderResult::OutOfDate
    }

    pub fn set_effect(&mut self, on: bool)
    {
        self.effect_on = on;
    }

    pub fn toggle_effect(&mut self)
    {
        self.effect_on = !self.effect_on;
    }

    pub fn next_scene(&mut self) -> anyhow::Result<usize>
    {
        let next_index = (self.scene_index + 1) % self.scenes.len();

        let ident = match self.scenes.get(next_index)
        {
            Some(scene) => scene.ident.clone(),
            _ => return Err(anyhow::Error::msg("No scenes")),
        };

        self.switch_scene(&ident)
    }
}
