#![allow(unused)]


use image::RgbaImage;


pub struct Image(RgbaImage);
impl Image
{
    pub fn new(bytes: &[u8]) -> anyhow::Result<Self>
    {
        Ok(Self(image::load_from_memory(bytes)?.to_rgba8()))
    }
}


impl std::ops::Deref for Image
{
    type Target = RgbaImage;

    fn deref(&self) -> &Self::Target
    {
        &self.0
    }
}


pub struct AsocTexture<'a>
{
    texture: Texture,
    image: MaybeOwned<'a, Image>,
}


pub struct Texture
{
    pub texture: wgpu::Texture,
    pub extent: wgpu::Extent3d,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}


impl Texture
{
    pub fn new(device: &wgpu::Device, extent: wgpu::Extent3d) -> Self
    {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("texture"),
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Texture Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        Self {
            texture,
            extent,
            view,
            sampler,
        }
    }

    pub fn write_texture(&self, queue: &wgpu::Queue, data: &Image)
    {
        let texture = wgpu::TexelCopyTextureInfo {
            texture: &self.texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        };

        let data_layout = wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * self.extent.width),
            rows_per_image: Some(self.extent.height),
        };

        queue.write_texture(texture, data, data_layout, self.extent);
    }
}


impl<'a> AsocTexture<'a>
{
    pub fn new(texture: Texture, image: MaybeOwned<'a, Image>) -> Self
    {
        Self { texture, image }
    }

    pub fn from_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        image: impl Into<MaybeOwned<'a, Image>>,
    ) -> Self
    {
        let image = image.into();
        let dims = image.dimensions();

        let extent = wgpu::Extent3d {
            width: dims.0,
            height: dims.1,
            depth_or_array_layers: 1,
        };

        let texture = Texture::new(device, extent);
        texture.write_texture(queue, &image);
        Self::new(texture, image)
    }

    pub fn texture(&self) -> &Texture
    {
        &self.texture
    }
}


impl<'a> From<AsocTexture<'a>> for Texture
{
    fn from(value: AsocTexture<'a>) -> Self
    {
        value.texture
    }
}


pub struct TextureBindGroupLayout
{
    layout: wgpu::BindGroupLayout,
}


impl TextureBindGroupLayout
{
    pub fn new(device: &wgpu::Device) -> Self
    {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("texture_bind_group_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        Self { layout }
    }

    pub fn create_bind_group(&self, device: &wgpu::Device, texture: &Texture) -> wgpu::BindGroup
    {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("texture_bind_group"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
            ],
        })
    }
}


impl std::ops::Deref for TextureBindGroupLayout
{
    type Target = wgpu::BindGroupLayout;

    fn deref(&self) -> &Self::Target
    {
        &self.layout
    }
}


pub enum MaybeOwned<'a, T>
{
    Owned(T),
    Borrowed(&'a T),
}


impl<'a, T> AsRef<T> for MaybeOwned<'a, T>
{
    fn as_ref(&self) -> &T
    {
        match self
        {
            MaybeOwned::Owned(val) => val,
            MaybeOwned::Borrowed(val) => val,
        }
    }
}


impl<'a, T> std::ops::Deref for MaybeOwned<'a, T>
{
    type Target = T;

    fn deref(&self) -> &T
    {
        self.as_ref()
    }
}


impl<T> From<T> for MaybeOwned<'static, T>
{
    fn from(value: T) -> Self
    {
        Self::Owned(value)
    }
}


impl<'a, T> From<&'a T> for MaybeOwned<'a, T>
{
    fn from(value: &'a T) -> Self
    {
        Self::Borrowed(value)
    }
}
