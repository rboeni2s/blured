mod renderer_impl;


use crate::service::{
    application::AppEvent,
    renderer::renderer_impl::RendererImpl,
    wlclient::WindowHandle,
};
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
}


impl SimpleDispatch<AppEvent> for Renderer {}
