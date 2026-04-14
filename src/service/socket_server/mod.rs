use crate::service::{
    application::{AppEvent, Application},
    renderer::Renderer,
};
use blured_ipc::{FromToJson, SOCKET_ADDR, msg::*};
use keep::Keep;
use plug::prelude::*;
use std::{
    io::{BufRead, BufReader, BufWriter, ErrorKind, Write},
    os::unix::net::{UnixListener, UnixStream},
    sync::atomic::{AtomicBool, Ordering},
    thread::JoinHandle,
    time::Duration,
};


#[service]
pub struct IpcSocket<AppEvent>
{
    #[layer]
    app: Application,

    #[layer]
    renderer: Renderer,

    #[value = None.into()]
    socket: Keep<Option<UnixListener>>,

    #[value = AtomicBool::new(false)]
    socket_shutdown: AtomicBool,

    #[value = None.into()]
    socket_task: Keep<Option<JoinHandle<()>>>,
}


#[derive(Clone)]
struct ClientData
{
    ipc: Layer<IpcSocket>,
    app: Layer<Application>,
    renderer: Layer<Renderer>,
}


impl IpcSocket
{
    fn discover_clients_task(client_data: ClientData)
    {
        if let Some(socket) = client_data.ipc.socket.read().as_ref()
        {
            'outer: loop
            {
                match socket.accept()
                {
                    Ok((client, addr)) =>
                    {
                        let cd = client_data.clone();
                        let _ = std::thread::spawn(move || {
                            log::info!("Handling new ipc client {addr:?}");

                            if let Err(e) = Self::handle_client_task(client, cd)
                            {
                                log::warn!("Error during ipc client communication {e}");
                            }
                        });
                    }

                    Err(e) =>
                    {
                        match e.kind()
                        {
                            ErrorKind::WouldBlock =>
                            {
                                if client_data.ipc.socket_shutdown.load(Ordering::Relaxed)
                                {
                                    break 'outer;
                                }

                                std::thread::sleep(Duration::from_millis(30));
                            }

                            _ =>
                            {
                                log::error!(
                                    "Failed to accept incoming uds stream {e} ({})",
                                    e.kind()
                                )
                            }
                        }
                    }
                }
            }
        }
    }

    fn handle_client_task(client: UnixStream, cd: ClientData) -> anyhow::Result<()>
    {
        let mut reader = BufReader::new(client.try_clone()?);
        let mut writer = BufWriter::new(client);

        let mut buf = Vec::with_capacity(256);

        while reader.read_until(b'\0', &mut buf).is_ok()
        {
            if buf.is_empty()
            {
                // Connection was closed
                break;
            }

            let message = Message::from_json_bytes(&buf[..buf.len().saturating_sub(1)])?;
            buf.clear();

            let instance = message.instance; //HACK: Fake the instance for now...


            let status = match message.action
            {
                Action::JumpToScene(scene) =>
                {
                    match cd.renderer.switch_scene(&scene)
                    {
                        Ok(_) =>
                        {
                            cd.app.reset_slideshow_timer();
                            Status::Ok(OkResponse::SwitchedScene(scene))
                        }
                        Err(_) => Status::Err(ErrResponse::NoSuchScene(scene)),
                    }
                }

                Action::NextScene =>
                {
                    match cd.renderer.next_scene()
                    {
                        Ok(name) =>
                        {
                            cd.app.reset_slideshow_timer();
                            Status::Ok(OkResponse::SwitchedScene(name))
                        }
                        Err(e) => Status::Err(ErrResponse::Error(e.to_string())),
                    }
                }

                Action::SetEffectOn(on) =>
                {
                    match cd.renderer.set_effect_on(on)
                    {
                        Ok(_) => Status::Ok(OkResponse::SetEffectOn(on)),
                        Err(e) => Status::Err(ErrResponse::Error(e.to_string())),
                    }
                }

                Action::ToggleEffect =>
                {
                    match cd.renderer.toggle_effect()
                    {
                        Ok(on) => Status::Ok(OkResponse::SetEffectOn(on)),
                        Err(e) => Status::Err(ErrResponse::Error(e.to_string())),
                    }
                }

                Action::SetSlideshowOn(on) =>
                {
                    cd.app.set_slideshow_active(on);
                    Status::Ok(OkResponse::Ok)
                }
            };

            let response = Response { instance, status }.to_json_bytes()?;
            writer.write_all(&response)?;
            writer.write_all(b"\0")?;
            writer.flush()?;
        }

        Ok(())
    }
}


impl SimpleDispatch<AppEvent> for IpcSocket
{
    fn simple_dispatch(&self, event: &AppEvent)
    {
        match event
        {
            AppEvent::Init(reg) =>
            {
                match UnixListener::bind(SOCKET_ADDR)
                {
                    Ok(listener) =>
                    {
                        listener.set_nonblocking(true).unwrap(); // This should not fail
                        self.socket.write(Some(listener));

                        let client_data = ClientData {
                            ipc: reg.get_unchecked(),
                            app: self.app.clone(),
                            renderer: self.renderer.clone(),
                        };

                        let task = std::thread::spawn(|| Self::discover_clients_task(client_data));
                        self.socket_task.write(Some(task));
                        log::info!("Opened new ipc server at {SOCKET_ADDR:?}");
                    }

                    Err(e) =>
                    {
                        log::warn!(
                            "Failed to bind to {SOCKET_ADDR:?}. Is another Blured instance running? {e}"
                        )
                    }
                }
            }

            AppEvent::Quit =>
            {
                self.socket_shutdown.store(true, Ordering::Relaxed);

                if let Some(task) = self.socket_task.swap(None).as_ref()
                {
                    while !task.is_finished()
                    {
                        std::hint::spin_loop();
                    }

                    _ = std::fs::remove_file(SOCKET_ADDR);
                    log::info!("Ipc Client discovery was shut down");
                }
            }
        }
    }
}
