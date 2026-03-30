use crate::{
    scene_desc::{Effect, ImageFit, ImageSceneDesc},
    service::{config::color::Color, renderer::image_scene::EffectParams},
};
use anyhow::Context;
use std::{path::PathBuf, time::Duration};


#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SlideshowInterval
{
    Sec(u32),
    Min(u32),
}


impl Default for SlideshowInterval
{
    fn default() -> Self
    {
        Self::Sec(30)
    }
}


impl From<SlideshowInterval> for Duration
{
    fn from(value: SlideshowInterval) -> Self
    {
        match value
        {
            SlideshowInterval::Sec(s) => Duration::from_secs(s as u64),
            SlideshowInterval::Min(m) => Duration::from_mins(m as u64),
        }
    }
}


#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImageSource
{
    #[default]
    Builtin,
    Path(String),
}


#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Monitor
{
    #[default]
    Auto,
    Named(String),
}


#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct AppConfig
{
    pub transition_time: f32,
    pub slideshow: bool,
    pub slideshow_interval: SlideshowInterval,
    pub scene: Vec<SceneConfig>,
}


impl AppConfig
{
    pub fn verify(&mut self) -> anyhow::Result<()>
    {
        for scene in &mut self.scene
        {
            if let ImageSource::Path(p) = &mut scene.image_source
            {
                let expanded = shellexpand::full(p)
                    .context(format!("Failed to expand: {p:?}"))?
                    .to_string();

                if !PathBuf::from(&expanded).is_file()
                {
                    return Err(anyhow::Error::msg(format!("{expanded:?} is not a file")));
                }

                *p = expanded;
            }

            if let Effect::Custom(p) = &mut scene.effect
            {
                let expanded = shellexpand::full(p)
                    .context(format!("Failed to expand: {p:?}"))?
                    .to_string();

                if !PathBuf::from(&expanded).is_file()
                {
                    return Err(anyhow::Error::msg(format!("{expanded:?} is not a file")));
                }

                *p = expanded;
            }
        }

        Ok(())
    }
}


impl Default for AppConfig
{
    fn default() -> Self
    {
        Self {
            transition_time: 0.2,
            scene: vec![SceneConfig::default()],
            slideshow: false,
            slideshow_interval: SlideshowInterval::default(),
        }
    }
}


#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "snake_case")]
pub struct SceneConfig
{
    pub name: String,
    pub image_source: ImageSource,
    pub image_fit: ImageFit,
    pub background_color: Color,
    pub effect: Effect,
    pub effect_strength: f32,
    pub dynamic: bool,
    pub effect_params: EffectParams,
}


impl Default for SceneConfig
{
    fn default() -> Self
    {
        Self {
            name: "builtin".into(),
            image_source: ImageSource::default(),
            image_fit: ImageFit::default(),
            background_color: Color::default(),
            effect: Effect::Neuro(Default::default()),
            effect_strength: 50.0,
            dynamic: false,
            effect_params: EffectParams::default(),
        }
    }
}


impl From<SceneConfig> for ImageSceneDesc
{
    fn from(scene: SceneConfig) -> Self
    {
        let image_source = match scene.image_source
        {
            ImageSource::Builtin => ImageSceneDesc::DEFAULT_IMAGE.to_vec(),
            ImageSource::Path(p) =>
            {
                std::fs::read(&p).unwrap_or_else(|_| {
                    log::error!("Failed to read {p}, defaulting to default wallpaper");
                    ImageSceneDesc::DEFAULT_IMAGE.to_vec()
                })
            }
        };

        ImageSceneDesc {
            ident: scene.name,
            image_source,
            image_fit: scene.image_fit,
            background: scene.background_color.into(),
            dynamic: scene.dynamic,
            effect_params: scene.effect_params,
            effect_strength: scene.effect_strength,
            effect: scene.effect,
        }
    }
}
