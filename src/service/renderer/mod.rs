mod buffer;
mod camera;
mod image_scene;
mod pipelines;
mod renderer_impl;
mod texture;


use std::{
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

use crate::service::{
    application::AppEvent,
    renderer::renderer_impl::RendererImpl,
    wlclient::WindowHandle,
};
use anyhow::Context;
use keep::Keep;
use plug::prelude::*;


#[service]
pub struct Renderer<AppEvent>
{
    #[value = None.into()]
    renderer: Keep<Option<RendererImpl>>,

    #[value = AtomicBool::new(false)]
    out_of_date: AtomicBool,
}


impl Renderer
{
    pub fn init(&self, window_handle: WindowHandle) -> anyhow::Result<()>
    {
        self.renderer.write(Some(RendererImpl::new(
            window_handle,
            &[
                // ImageSceneDesc {
                // ident: "w1".to_string(),
                // image_source: include_bytes!("../../../../../Bilder/Wallpaper/path.jpg").to_vec(),
                // image_fit: ImageFit::default(),
                // ..Default::default()
                // }
            ],
        )?));

        self.out_of_date.store(true, Ordering::Relaxed);
        Ok(())
    }

    pub fn dispatch(&self, _delta: Duration) -> anyhow::Result<()>
    {
        // Render if the surface is out of date
        if self.out_of_date.load(Ordering::Acquire)
        {
            self.render()?;

            // Try to mark the surface as not out of date
            let _ = self.out_of_date.compare_exchange(
                true,
                false,
                Ordering::Release,
                Ordering::Relaxed,
            );
        }

        Ok(())
    }

    fn render(&self) -> anyhow::Result<()>
    {
        self.renderer
            .read()
            .as_ref()
            .as_ref()
            .context("Trying to render while no renderer has been initialized")?
            .render()
    }
}


impl SimpleDispatch<AppEvent> for Renderer
{
    fn simple_dispatch(&self, event: &AppEvent)
    {
        if let AppEvent::Quit = event
            && let Some(renderer) = self.renderer.read().as_ref()
        {
            renderer.destroy_surface();
        }
    }
}
