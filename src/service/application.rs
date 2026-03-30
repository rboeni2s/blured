use std::{
    sync::nonpoison::Mutex,
    time::{Duration, Instant},
};

use keep::Keep;
use plug::prelude::*;

use crate::service::{config::Config, renderer::Renderer, wlclient::WlClient};


#[derive(Clone)]
pub enum AppEvent
{
    Init(Guard<Registry<Self>>),
    Quit,
}


struct SlideShowData
{
    interval: Duration,
    active: bool,
    time: Instant,
}


impl SlideShowData
{
    pub fn manage(&mut self, app: &Application)
    {
        if self.active && self.time.elapsed() > self.interval
        {
            self.time = Instant::now();
            if let Err(e) = app.renderer.next_scene()
            {
                log::error!("Failed to switch to the next scene: {e}");
            }
        }
    }
}


impl Default for SlideShowData
{
    fn default() -> Self
    {
        Self {
            interval: Default::default(),
            active: Default::default(),
            time: Instant::now(),
        }
    }
}


#[service]
pub struct Application<AppEvent>
{
    #[layer]
    config: Config,

    #[layer]
    wl_client: WlClient,

    #[layer]
    renderer: Renderer,

    #[value = false.into()]
    should_quit: Keep<bool>,

    #[value = Mutex::new(SlideShowData::default())]
    slideshow: Mutex<SlideShowData>,
}


impl Application
{
    /// Runs the application
    pub fn run(&self, registry: Guard<Registry<AppEvent>>) -> anyhow::Result<()>
    {
        let mut ret: anyhow::Result<()> = Ok(());

        // Initialize the application
        if let Err(e) = self.init(&registry)
        {
            log::error!("Error during initialization {e}");
            ret = Err(e);
        }
        // Only start dispatching if the initialization succeeded
        else
        {
            if let Err(e) = self.dispatch()
            {
                log::error!("Error during dispatch: {e}");
                ret = Err(e);
            }

            self.quit();
        }

        // Finally destroy the application
        if let Err(e) = self.destroy(registry)
        {
            log::error!("Error during teardown: {e}");
            ret = Err(e);
        }

        ret
    }

    /// Signal the application to quit
    pub fn quit(&self)
    {
        self.should_quit.write(true);
    }

    fn dispatch(&self) -> anyhow::Result<()>
    {
        const FRAME_TIME_TARGET_MS: u64 = 33;

        let mut last_frame_time;
        let mut frame_start = Instant::now();

        while !*self.should_quit.read()
        {
            last_frame_time = frame_start.elapsed();
            frame_start = Instant::now();

            self.slideshow.lock().manage(self);
            self.wl_client.dispatch(last_frame_time)?;
            self.renderer.dispatch(last_frame_time)?;

            std::thread::sleep(
                Duration::from_millis(FRAME_TIME_TARGET_MS).saturating_sub(frame_start.elapsed()),
            );
        }

        Ok(())
    }

    fn init(&self, registry: &Guard<Registry<AppEvent>>) -> Result<(), anyhow::Error>
    {
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

        // init slideshow from config
        self.apply_slideshow_config();

        Ok(())
    }

    fn apply_slideshow_config(&self)
    {
        let mut slideshow = self.slideshow.lock();
        let config = self.config.get_active_config();
        slideshow.active = config.slideshow;
        slideshow.interval = config.slideshow_interval.clone().into();
        slideshow.time = Instant::now();
    }

    fn destroy(&self, registry: Guard<Registry<AppEvent>>) -> anyhow::Result<()>
    {
        log::debug!("Quitting application...");
        registry.dispatch(&AppEvent::Quit);
        log::debug!("Quitting done!");
        Ok(())
    }
}


impl SimpleDispatch<AppEvent> for Application {}
