use anyhow::Context;

use crate::{
    scene_desc::{Effect, ImageFit},
    service::{config::color::Color, renderer::image_scene::EffectParams},
};
use std::path::PathBuf;


#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ImageSource
{
    #[default]
    Builtin,
    Path(String),
}


#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct AppConfig
{
    pub transition_time: f32,
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
        }
    }
}


#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct SceneConfig
{
    pub(crate) name: String,
    pub(crate) image_source: ImageSource,
    pub(crate) image_fit: ImageFit,
    pub(crate) background_color: Color,
    pub(crate) effect: Effect,
    pub(crate) effect_strength: f32,
    pub(crate) dynamic: bool,
    pub(crate) effect_params: EffectParams,
}


impl Default for SceneConfig
{
    fn default() -> Self
    {
        Self {
            name: "Builtin".into(),
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
