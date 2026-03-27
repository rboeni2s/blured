pub mod application;
pub mod config;
pub mod renderer;
pub mod wlclient;

use crate::service::{
    application::{AppEvent, Application},
    config::Config,
    renderer::Renderer,
    wlclient::WlClient,
};
use plug::prelude::*;


/// Builds a service registry.
///
/// Returns a result in case building a registry includes
/// conditional services in the future that could result in the failure to
/// construct a registry.
pub fn build_reg() -> anyhow::Result<Registry<AppEvent>>
{
    Ok(build_reg!(Application, WlClient, Renderer, Config))
}
