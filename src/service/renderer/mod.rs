mod renderer_impl;


use std::time::Duration;

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
}


impl Renderer
{
    pub fn init(&self, window_handle: WindowHandle) -> anyhow::Result<()>
    {
        self.renderer.write(Some(RendererImpl::new(window_handle)?));
        Ok(())
    }

    pub fn dispatch(&self, _delta: Duration) -> anyhow::Result<()>
    {
        self.render()
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
