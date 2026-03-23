mod buffer;
mod camera;
mod image_scene;
mod pipelines;
mod renderer_impl;
mod texture;


use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        nonpoison::Mutex,
    },
    time::Duration,
};

use crate::service::{
    application::AppEvent,
    renderer::{
        image_scene::{ImageFit, ImageSceneDesc},
        renderer_impl::RendererImpl,
    },
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
    #[value = None.into()]
    renderer: Mutex<Option<RendererImpl>>,

    #[value = AtomicBool::new(false)]
    out_of_date: AtomicBool,
}


impl Renderer
{
    pub fn init(&self, window_handle: WindowHandle) -> anyhow::Result<()>
    {
        *self.renderer.lock() = Some(RendererImpl::new(
            window_handle,
            &[
                ImageSceneDesc {
                    ident: "w1".to_string(),
                    image_source: include_bytes!("../../../../../Bilder/Wallpaper/path.jpg")
                        .to_vec(),
                    image_fit: ImageFit::default(),
                    ..Default::default()
                },
                ImageSceneDesc {
                    ident: "w2".to_string(),
                    image_source: include_bytes!("../../../textures/swirls.jpg").to_vec(),
                    image_fit: ImageFit::Stretch,
                    ..Default::default()
                },
            ],
        )?);

        self.out_of_date.store(true, Ordering::Relaxed);
        Ok(())
    }

    pub fn dispatch(&self, _delta: Duration) -> anyhow::Result<()>
    {
        // Render if the surface is out of date
        if self.out_of_date.load(Ordering::Acquire)
        {
            // Don't rerender the scene if it is not dynamic
            if self.render()? == RenderResult::Clean
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

    fn render(&self) -> anyhow::Result<RenderResult>
    {
        self.renderer
            .lock()
            .as_mut()
            .context("Trying to render while no renderer has been initialized")?
            .render()
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
