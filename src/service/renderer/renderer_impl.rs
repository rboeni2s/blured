use anyhow::Context;

use crate::service::wlclient::WindowHandle;


#[allow(unused)]
pub struct RendererImpl
{
    instance: wgpu::Instance,
    surface: wgpu::Surface<'static>,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    width: u32,
    height: u32,
}


impl RendererImpl
{
    pub fn new(window_handle: WindowHandle) -> anyhow::Result<Self>
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

        let surface_texture = match surface.get_current_texture()
        {
            wgpu::CurrentSurfaceTexture::Success(texture) => texture,
            e =>
            {
                return Err(anyhow::Error::msg(format!(
                    "Failed to fetch current surface texture due to: {e:?}"
                )));
            }
        };

        let texture_view = surface_texture.texture.create_view(&Default::default());
        let mut encoder = device.create_command_encoder(&Default::default());

        {
            let _renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                ..Default::default()
            });
        }

        queue.submit(Some(encoder.finish()));
        surface_texture.present();

        Ok(Self {
            instance,
            surface,
            adapter,
            device,
            queue,
            width,
            height,
        })
    }
}
