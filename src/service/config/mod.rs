pub mod color;
pub mod params;


use crate::service::{application::AppEvent, config::params::AppConfig};
use anyhow::Context;
use plug::prelude::*;
use plugmap::PlugMap;
use std::path::PathBuf;


const CONFIG_DIR: &str = "~/.config/blured";


#[service]
pub struct Config<AppEvent>
{
    #[default]
    configs: PlugMap<String, AppConfig>,
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

                            if !name.is_empty() && ext.to_lowercase() == "kdl"
                            {
                                log::info!("Reading config from: {path:?}");

                                let data = std::fs::read_to_string(&path)
                                    .context(format!("Failed to read {path:?}"))?;

                                match serde_kdl2::from_str::<AppConfig>(&data)
                                {
                                    Ok(mut conf) =>
                                    {
                                        let name = name.to_lowercase();
                                        if let Err(e) = conf.verify()
                                        {
                                            log::error!("Error while parsing config {path:?}: {e}");
                                        }
                                        log::info!("Found config {name:?}: {conf:#?}");
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
}


impl SimpleDispatch<AppEvent> for Config
{
    fn simple_dispatch(&self, event: &AppEvent)
    {
        if let AppEvent::Init(_reg) = event
        {
            if let Err(e) = self.update_configs()
            {
                log::error!("Failed to read config: {e}. Proceeding with incomplete config...");
            }
        }
    }
}
