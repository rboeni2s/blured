use crate::service::application::AppEvent;
use plug::prelude::*;


#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppConfig
{
    transition_time: f32,
    scenes: Vec<SceneConfig>,
}


#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SceneConfig {}


#[service]
pub struct Config<AppEvent> {}
impl SimpleDispatch<AppEvent> for Config {}
