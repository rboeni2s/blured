use crate::service::application::{AppEvent, Application};
use anyhow::Context;
use keep::Keep;
use plug::prelude::*;
use raw_window_handle::{
    RawDisplayHandle,
    RawWindowHandle,
    WaylandDisplayHandle,
    WaylandWindowHandle,
};
use std::{
    ptr::NonNull,
    sync::nonpoison::Mutex,
    time::{Duration, Instant},
};
use wayland_client::{
    Connection,
    Dispatch,
    EventQueue,
    Proxy,
    QueueHandle,
    protocol::{
        wl_buffer,
        wl_compositor,
        wl_display::WlDisplay,
        wl_registry::{self, WlRegistry},
        wl_shm,
        wl_shm_pool,
        wl_surface,
    },
};


use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1,
    zwlr_layer_surface_v1::{self},
};


/// Wraps a `RawDisplayHandle`, `RawWindowHandle` and surface size.
pub struct WindowHandle
{
    pub display_handle: RawDisplayHandle,
    pub window_handle: RawWindowHandle,
    pub surface_size: (u32, u32),
}


#[service]
pub struct WlClient<AppEvent>
{
    #[default]
    dispatcher: Mutex<WlDispatcher>,

    #[value = None.into()]
    wl_conn: Keep<Option<Mutex<WlConnection>>>,
}


impl WlClient
{
    /// Returns `Ok(false)` if there where no pending events
    pub fn dispatch(&self, _delta: Duration) -> anyhow::Result<bool>
    {
        let conn = self.wl_conn.read();
        let mut conn = conn.as_ref().as_ref().unwrap().lock();

        let event_count = conn
            .event_queue
            .dispatch_pending(&mut self.dispatcher.lock())?;

        if event_count > 0
        {
            return Ok(true);
        }

        conn.event_queue.flush()?;

        if let Some(guard) = conn.connection.prepare_read()
            && guard.read().is_ok()
        {
            let event_count = conn
                .event_queue
                .dispatch_pending(&mut self.dispatcher.lock())?;

            return Ok(event_count != 0);
        }

        Ok(false)
    }

    pub fn window_handle(&self) -> anyhow::Result<WindowHandle>
    {
        let dispatcher = self.dispatcher.lock();

        Ok(WindowHandle {
            display_handle: self
                .wl_conn
                .read()
                .as_ref()
                .as_ref()
                .context("Wayland session not yet connected")?
                .lock()
                .raw_display_handle()?,
            window_handle: dispatcher.raw_window_handle()?,
            surface_size: dispatcher.surface_size,
        })
    }
}


impl SimpleDispatch<AppEvent> for WlClient
{
    fn simple_dispatch(&self, event: &AppEvent)
    {
        // Setup a wayland client in init
        if let AppEvent::Init(reg) = event
        {
            // Connect to the wayland server
            let connection = match Connection::connect_to_env()
            {
                Ok(connection) => connection,
                Err(e) =>
                {
                    log::error!("Failed to open a wayland connection: {e}");
                    reg.get_unchecked::<Application>().quit();
                    return;
                }
            };

            let display = connection.display();
            let event_queue = connection.new_event_queue();
            let event_queue_handle = event_queue.handle();
            let registry = display.get_registry(&event_queue_handle, ());

            let mut wl_connection = WlConnection {
                connection,
                display,
                event_queue,
                event_queue_handle,
                registry,
            };

            if let Err(e) = wl_connection
                .event_queue
                .roundtrip(&mut self.dispatcher.lock())
            {
                log::error!("First roundtrip to the wayland server failed: {e}");
                return;
            }

            {
                let mut missing_protocol = false;
                let mut dispatcher = self.dispatcher.lock();

                if dispatcher.compositor.is_none()
                {
                    missing_protocol = true;
                    log::error!("Missing wayland protocol wl_compositor");
                }

                if dispatcher.shm.is_none()
                {
                    missing_protocol = true;
                    log::error!("Missing wayland protocol wl_compositor");
                }

                if dispatcher.layer_shell.is_none()
                {
                    missing_protocol = true;
                    log::error!("Missing wayland protocol zwlr_layer_shell_v1");
                }

                // Request a quit if protocols where missing...
                if missing_protocol
                {
                    reg.get_unchecked::<Application>().quit();
                }

                dispatcher.create_surface(&wl_connection.event_queue_handle);

                // Block until the surface is configured or a response timed out
                let now = Instant::now();
                while dispatcher.surface_size == (0, 0) && now.elapsed() < Duration::from_secs(5)
                {
                    _ = wl_connection.event_queue.blocking_dispatch(&mut dispatcher);
                }
            }

            self.wl_conn.write(Some(Mutex::new(wl_connection)));
        }
    }
}


#[allow(unused)]
struct WlConnection
{
    connection: Connection,
    display: WlDisplay,
    event_queue: EventQueue<WlDispatcher>,
    event_queue_handle: QueueHandle<WlDispatcher>,
    registry: WlRegistry,
}


impl WlConnection
{
    fn raw_display_handle(&self) -> anyhow::Result<RawDisplayHandle>
    {
        Ok(RawDisplayHandle::Wayland(WaylandDisplayHandle::new(
            NonNull::new(self.display.id().as_ptr() as *mut _)
                .context("Wayland display handle was null")?,
        )))
    }
}


#[derive(Default)]
struct WlDispatcher
{
    // After Initialization shm, compositor, layer_shell and buffer_file can be assumed to be Some(_)
    shm: Option<wl_shm::WlShm>,
    compositor: Option<wl_compositor::WlCompositor>,
    layer_shell: Option<zwlr_layer_shell_v1::ZwlrLayerShellV1>,

    layer_surface: Option<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1>,
    surface: Option<wl_surface::WlSurface>,
    surface_size: (u32, u32),
}


impl WlDispatcher
{
    fn create_surface(&mut self, queue_handle: &QueueHandle<Self>)
    {
        log::debug!("Creating surface");

        let provided_surface = self
            .compositor
            .as_ref()
            .unwrap()
            .create_surface(queue_handle, ());

        let layer_surface = self.layer_shell.as_ref().unwrap().get_layer_surface(
            &provided_surface,
            None,
            zwlr_layer_shell_v1::Layer::Background,
            "wallpaper".into(),
            queue_handle,
            (),
        );

        layer_surface.set_size(0, 0);
        layer_surface.set_anchor(zwlr_layer_surface_v1::Anchor::all());
        layer_surface
            .set_keyboard_interactivity(zwlr_layer_surface_v1::KeyboardInteractivity::None);
        layer_surface.set_exclusive_zone(-1);
        layer_surface.set_margin(0, 0, 0, 0);
        provided_surface.commit();

        self.surface = Some(provided_surface);
        self.layer_surface = Some(layer_surface);
    }

    fn raw_window_handle(&self) -> anyhow::Result<RawWindowHandle>
    {
        Ok(RawWindowHandle::Wayland(WaylandWindowHandle::new(
            NonNull::new(
                self.surface
                    .as_ref()
                    .context("Wayland surface not yet initialized")?
                    .id()
                    .as_ptr() as *mut _,
            )
            .context("Wayland surface pointer was unexpectedly null")?,
        )))
    }
}


impl Dispatch<wl_registry::WlRegistry, ()> for WlDispatcher
{
    fn event(
        state: &mut Self,
        proxy: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _data: &(),
        _conn: &wayland_client::Connection,
        queue_handle: &wayland_client::QueueHandle<Self>,
    )
    {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            log::debug!("Found Global [{name}] {interface} (v{version})");

            match &interface[..]
            {
                "wl_shm" =>
                {
                    state.shm = Some(proxy.bind(name, version, queue_handle, ()));
                }

                "wl_compositor" =>
                {
                    state.compositor = Some(proxy.bind(name, version, queue_handle, ()));
                }

                "zwlr_layer_shell_v1" =>
                {
                    state.layer_shell = Some(proxy.bind(name, version, queue_handle, ()));
                }

                _ => (),
            }
        }
    }
}


impl wayland_client::Dispatch<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1, ()> for WlDispatcher
{
    fn event(
        state: &mut Self,
        proxy: &zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
        event: zwlr_layer_surface_v1::Event,
        _data: &(),
        _conn: &wayland_client::Connection,
        _queue_handle: &wayland_client::QueueHandle<Self>,
    )
    {
        if let zwlr_layer_surface_v1::Event::Configure {
            serial,
            width,
            height,
        } = event
        {
            log::debug!("zwlr_layer_surface_v1::configure received: {event:?}");
            state.surface_size = (width, height);
            proxy.ack_configure(serial);
        }
    }
}


wayland_client::delegate_noop!(WlDispatcher: ignore wl_shm::WlShm);
wayland_client::delegate_noop!(WlDispatcher: ignore wl_shm_pool::WlShmPool);
wayland_client::delegate_noop!(WlDispatcher: ignore wl_buffer::WlBuffer);
wayland_client::delegate_noop!(WlDispatcher: ignore wl_compositor::WlCompositor);
wayland_client::delegate_noop!(WlDispatcher: ignore wl_surface::WlSurface);
wayland_client::delegate_noop!(WlDispatcher: ignore zwlr_layer_shell_v1::ZwlrLayerShellV1);
