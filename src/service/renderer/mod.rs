pub mod buffer;
pub mod camera;
pub mod image_scene;
pub mod pipelines;
pub mod renderer_impl;
pub mod texture;


use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        nonpoison::Mutex,
    },
    time::Duration,
};

use crate::service::config::Config;
use crate::service::{
    application::AppEvent,
    renderer::renderer_impl::RendererImpl,
    wlclient::WindowHandle,
};
use anyhow::Context;
use plug::prelude::*;


#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RenderResult
{
    Clean,
    OutOfDate,
}


#[service]
pub struct Renderer<AppEvent>
{
    #[layer]
    config: Config,

    #[value = None.into()]
    renderer: Mutex<Option<RendererImpl>>,

    #[value = AtomicBool::new(false)]
    out_of_date: AtomicBool,
}


impl Renderer
{
    pub fn init(&self, window_handle: WindowHandle) -> anyhow::Result<()>
    {
        let scenes = self.config.get_scene_desc();
        *self.renderer.lock() = Some(RendererImpl::new(window_handle, &scenes)?);
        self.out_of_date.store(true, Ordering::Relaxed);
        Ok(())
    }

    pub fn dispatch(&self, delta: Duration) -> anyhow::Result<()>
    {
        // Render if the surface is out of date
        if self.out_of_date.load(Ordering::Acquire)
        {
            // Don't rerender the scene if it is not out of date
            if self.render(delta)? == RenderResult::Clean
            {
                // Try to mark the surface as not out of date
                let _ = self.out_of_date.compare_exchange(
                    true,
                    false,
                    Ordering::Release,
                    Ordering::Relaxed,
                );
            }
        }

        Ok(())
    }

    pub fn next_scene(&self) -> anyhow::Result<String>
    {
        let ident = self
            .renderer
            .lock()
            .as_mut()
            .context("Renderer was not initialized")?
            .next_scene()?;

        self.out_of_date.store(true, Ordering::Relaxed);
        Ok(ident)
    }

    pub fn switch_scene(&self, ident: &str) -> anyhow::Result<()>
    {
        self.renderer
            .lock()
            .as_mut()
            .context("Renderer was not initialized")?
            .switch_scene(ident)?;

        self.out_of_date.store(true, Ordering::Relaxed);
        Ok(())
    }

    pub fn set_effect_on(&self, on: bool) -> anyhow::Result<()>
    {
        self.renderer
            .lock()
            .as_mut()
            .context("Renderer was not initialized")?
            .set_effect(on);

        self.out_of_date.store(true, Ordering::Relaxed);

        Ok(())
    }

    pub fn toggle_effect(&self) -> anyhow::Result<bool>
    {
        let on = self
            .renderer
            .lock()
            .as_mut()
            .context("Renderer was not initialized")?
            .toggle_effect();

        self.out_of_date.store(true, Ordering::Relaxed);

        Ok(on)
    }

    fn render(&self, delta: Duration) -> anyhow::Result<RenderResult>
    {
        self.renderer
            .lock()
            .as_mut()
            .context("Trying to render while no renderer has been initialized")?
            .render(delta)
    }
}


impl SimpleDispatch<AppEvent> for Renderer
{
    fn simple_dispatch(&self, event: &AppEvent)
    {
        if let AppEvent::Quit = event
            && let Some(renderer) = self.renderer.lock().as_mut()
        {
            renderer.destroy_surface();
        }
    }
}
