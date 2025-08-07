mod lambda;
mod log;
mod std;

use ::std::{net::SocketAddr, path::Path};
use clap::Parser;
use federated_server::{AccessToken, GraphLoader};
use graph_ref::GraphRef;
pub(crate) use log::*;

pub(crate) trait Args {
    fn listen_address(&self) -> Option<SocketAddr>;

    fn log_level(&self) -> LogLevel<'_>;

    fn fetch_method(&self) -> anyhow::Result<GraphLoader>;


    fn config_path(&self) -> Option<&Path>;

    fn hot_reload(&self) -> bool;

    fn log_style(&self) -> LogStyle;

    fn graph_ref(&self) -> Option<&GraphRef>;

    fn can_export_telemetry_to_platform(&self) -> bool {
        AccessToken::is_defined_in_env() && self.graph_ref().is_some()
    }

    fn grafbase_access_token(&self) -> anyhow::Result<Option<AccessToken>> {
        AccessToken::from_env().map_err(anyhow::Error::msg)
    }

}

pub(crate) fn parse() -> impl Args {
    cfg_if::cfg_if! {
        if #[cfg(feature = "lambda")] {
            lambda::Args::parse()
        } else {
            std::Args::parse()
        }
    }
}
