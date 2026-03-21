use std::{env, time::Duration};

use keep::Keep;
use plug::prelude::*;

use crate::service::{renderer::Renderer, wlclient::WlClient};


#[derive(Clone)]
pub enum AppEvent
{
    Init(Guard<Registry<Self>>),
    Quit,
}


#[service]
pub struct Application<AppEvent>
{
    #[layer]
    wl_client: WlClient,

    #[layer]
    renderer: Renderer,

    #[value = false.into()]
    should_quit: Keep<bool>,
}


impl Application
{
    /// Runs the application
    pub fn run(&self, registry: Guard<Registry<AppEvent>>) -> anyhow::Result<()>
    {
        // Setup env vars
        if env::var("WGPU_BACKEND").is_err()
        {
            unsafe { env::set_var("WGPU_BACKEND", "vulkan") };
        }


        ctrlc::set_handler({
            let should_quit = self.should_quit.clone();
            move || should_quit.write(true)
        })?;

        log::debug!("Initializing Application...");
        registry.dispatch(&AppEvent::Init(registry.clone()));
        log::debug!("Initialization done!");

        if *self.should_quit.read()
        {
            log::warn!(
                "Quit request received during initialization, the most likely cause of this is an error during initialization"
            );
        }

        log::debug!("Initializing renderer...");
        self.renderer.init(self.wl_client.window_handle()?)?;
        log::debug!("Renderer Initialized!");

        while !*self.should_quit.read()
        {
            // Dispatch wayland events and quit on error
            match self.wl_client.dispatch()
            {
                Err(e) =>
                {
                    log::error!("Error during wl_client dispatch: {e}");
                    self.quit();
                }

                Ok(false) => std::thread::sleep(Duration::from_millis(30)),
                _ => (),
            }
        }

        log::debug!("Quitting application...");
        registry.dispatch(&AppEvent::Quit);
        log::debug!("Quitting done!");

        Ok(())
    }

    /// Will signal the application to quit
    pub fn quit(&self)
    {
        self.should_quit.write(true);
    }
}


impl SimpleDispatch<AppEvent> for Application {}
