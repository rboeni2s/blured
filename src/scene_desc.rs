use crate::service::renderer::image_scene::EffectParams;
use crate::service::renderer::image_scene::ImageScene;
use crate::service::renderer::pipelines::EffectPipeline;
use crate::service::renderer::pipelines::ScenePipeline;
use keep::Guard;


#[allow(unused)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ImageFit
{
    Stretch,
    FillH,
    #[default]
    FillV,
    Original,
}


#[allow(unused)]
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Effect
{
    Blur(BlurSettings),
    Neuro(NeuroSettings),
    Custom(String),
}


#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct BlurSettings
{
    pub quality: f32,
    pub directions: f32,
}


impl Default for BlurSettings
{
    fn default() -> Self
    {
        Self {
            quality: 64.0,
            directions: 20.0,
        }
    }
}


#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct NeuroSettings
{
    pub scale: f32,
    pub speed: f32,
    pub dim: f32,
    pub ambient: f32,
}


impl Default for NeuroSettings
{
    fn default() -> Self
    {
        Self {
            scale: 2.8,
            speed: 0.4,
            dim: 17.0,
            ambient: 0.3,
        }
    }
}


impl From<&NeuroSettings> for EffectParams
{
    fn from(value: &NeuroSettings) -> Self
    {
        EffectParams {
            param_a: [value.scale, value.speed, value.dim, value.ambient],
            ..Default::default()
        }
    }
}


impl From<&BlurSettings> for EffectParams
{
    fn from(value: &BlurSettings) -> Self
    {
        EffectParams {
            param_a: [value.quality, value.directions, 0.0, 0.0],
            ..Default::default()
        }
    }
}


impl Effect
{
    pub fn fetch_pipeline(
        &self,
        device: &wgpu::Device,
        pipeline: &EffectPipeline,
        effect_params: &EffectParams,
    ) -> anyhow::Result<(Guard<wgpu::RenderPipeline>, EffectParams)>
    {
        Ok(match self
        {
            // Get guards to the shared builtin pipelines
            Effect::Blur(settings) => (pipeline.blur_pipeline.clone(), settings.into()),
            Effect::Neuro(settings) => (pipeline.neuro_pipeline.clone(), settings.into()),

            // Load a user supplied wgsl shader from disk
            Effect::Custom(path) =>
            {
                let data = std::fs::read_to_string(path)?;
                let scope_guard = device.push_error_scope(wgpu::ErrorFilter::Validation);

                let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("User Shader Module"),
                    source: wgpu::ShaderSource::Wgsl(data.into()),
                });

                if let Some(error) = pollster::block_on(scope_guard.pop())
                {
                    return Err(error.into());
                }

                (
                    pipeline.create_pipeline(device, &shader)?,
                    effect_params.clone(),
                )
            }
        })
    }

    /// Returns `true` if a builtin effect does require dynamic rendering
    /// and needs to overwrite the "dynamic" field in it's config.
    pub fn require_dynamic(&self) -> bool
    {
        matches!(self, Effect::Neuro(_))
    }
}


pub struct ImageSceneDesc
{
    pub ident: String,
    pub image_source: Vec<u8>,
    pub image_fit: ImageFit,
    pub background: [f32; 3],
    pub dynamic: bool,
    pub effect_params: EffectParams,
    pub effect_strength: f32,
    pub effect: Effect,
}


impl ImageSceneDesc
{
    pub const DEFAULT_IMAGE: &[u8] = include_bytes!("../textures/astro_miku.jpg");

    pub fn load(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene_pipeline: &ScenePipeline,
        effect_pipeline: &EffectPipeline,
        surface_size: (u32, u32),
    ) -> anyhow::Result<ImageScene>
    {
        ImageScene::new(
            self,
            device,
            queue,
            scene_pipeline,
            effect_pipeline,
            surface_size,
        )
    }
}


impl Default for ImageSceneDesc
{
    fn default() -> Self
    {
        Self {
            ident: "builtin".into(),
            image_source: Self::DEFAULT_IMAGE.to_vec(),
            image_fit: Default::default(),
            background: [0.055 * 0.5, 0.12 * 0.5, 0.2 * 0.5],
            dynamic: false,
            effect_params: EffectParams::default(),
            effect_strength: 50.0,
            effect: Effect::Neuro(NeuroSettings::default()),
        }
    }
}
