pub mod color;
pub mod params;


use crate::{
    scene_desc::ImageSceneDesc,
    service::{
        application::AppEvent,
        config::params::{AppConfig, Monitor},
    },
};
use anyhow::Context;
use keep::Keep;
use plug::prelude::*;
use plugmap::PlugMap;
use std::path::PathBuf;


const CONFIG_DIR: &str = "~/.config/blured";


#[service]
pub struct Config<AppEvent>
{
    #[default]
    configs: PlugMap<String, AppConfig>,

    #[value = Monitor::default().into()]
    monitor: Keep<Monitor>,

    #[value = "builtin".to_string().into()]
    active_config: Keep<String>,
}


impl Config
{
    pub fn update_configs(&self) -> anyhow::Result<()>
    {
        let path = shellexpand::full(CONFIG_DIR)
            .context(format!("Path expansion for {CONFIG_DIR:?} failed"))?;

        let path = PathBuf::from({
            let this: &str = &path;
            this
        })
        .canonicalize()
        .context(format!("{CONFIG_DIR:?} does not exist"))?;

        for conf in path
            .read_dir()
            .context(format!(
                "{CONFIG_DIR:?} does not exist or is not a directory"
            ))?
            .filter_map(|e| e.ok())
        {
            match conf.file_type()
            {
                Ok(t) =>
                {
                    match t.is_dir()
                    {
                        true =>
                        {
                            log::info!("Skipping directory: {:?} in config dir", conf.path())
                        }
                        false =>
                        {
                            let fname = conf.file_name().to_string_lossy().to_string();
                            let path = conf.path();
                            let (name, ext) = fname.rsplit_once(".").unwrap_or_default();

                            if !name.is_empty() && ext.to_lowercase() == "toml"
                            {
                                log::info!("Reading config from: {path:?}");

                                let data = std::fs::read_to_string(&path)
                                    .context(format!("Failed to read {path:?}"))?;

                                match toml::from_str::<AppConfig>(&data)
                                {
                                    Ok(mut conf) =>
                                    {
                                        let name = name.to_lowercase();
                                        if let Err(e) = conf.verify()
                                        {
                                            log::error!("Error while parsing config {path:?}: {e}");
                                        }
                                        log::info!("Found config {name:?}");
                                        log::debug!("Config {name:?} contents: {conf:?}");

                                        if self.active_config.read().as_str() == "builtin"
                                        {
                                            self.active_config.write(name.clone());
                                        }

                                        self.configs.insert(name, conf);
                                    }
                                    Err(e) =>
                                    {
                                        log::error!("Error while parsing config {path:?}: {e}");
                                        continue;
                                    }
                                }
                            }
                            else
                            {
                                log::info!("Skipping file: {path:?} in config dir");
                            }
                        }
                    }
                }

                Err(e) =>
                {
                    log::warn!(
                        "Skipping reading {:?} in config dir due to: {e}",
                        conf.file_name()
                    )
                }
            }
        }

        Ok(())
    }

    pub fn get_monitor(&self) -> Guard<Monitor>
    {
        self.monitor.read()
    }

    pub fn set_monitor(&self, monitor: Monitor)
    {
        self.monitor.write(monitor);
    }

    pub fn set_active_config(&self, name: impl Into<String>)
    {
        let name = name.into();
        log::info!("Set active config to: {name:?}");
        self.active_config.write(name);
    }

    pub fn get_config(&self, name: &String) -> Option<Guard<AppConfig>>
    {
        self.configs.get(&name)
    }

    pub fn get_active_config(&self) -> Guard<AppConfig>
    {
        let conf = &self.active_config.read();

        self.get_config(conf).unwrap_or_else(|| {
            log::warn!("No config named: {conf:?}");
            Guard::new(AppConfig::default())
        })
    }

    pub fn get_scene_desc(&self) -> Vec<ImageSceneDesc>
    {
        self.get_active_config()
            .scene
            .iter()
            .cloned()
            .map(ImageSceneDesc::from)
            .collect()
    }
}


impl SimpleDispatch<AppEvent> for Config
{
    fn simple_dispatch(&self, event: &AppEvent)
    {
        if let AppEvent::Init(_reg) = event
        {
            // println!("{}", toml::to_string(&AppConfig::default()).unwrap());

            if let Err(e) = self.update_configs()
            {
                log::error!("Failed to read config: {e}. Proceeding with incomplete config...");
            }

            if self
                .configs
                .insert("builtin".into(), AppConfig::default())
                .is_some()
            {
                log::warn!("Not builtin config named \"builtin\". Restored actual builtin config");
            }

            // Try and set the config name if it was passed as args
            if let Some(conf_name) = std::env::args().nth(1)
            {
                self.set_active_config(conf_name);
            }
        }
    }
}
