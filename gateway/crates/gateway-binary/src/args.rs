mod lambda;
mod log;
mod std;

use ::std::{net::SocketAddr, path::Path};

use clap::Parser;
use federated_server::GraphFetchMethod;
use gateway_config::Config;
use grafbase_telemetry::otel::layer::BoxedLayer;
pub(crate) use log::LogLevel;
use tracing::Subscriber;
use tracing_subscriber::registry::LookupSpan;

pub(crate) trait Args {
    fn listen_address(&self) -> Option<SocketAddr>;

    fn log_level(&self) -> Option<LogLevel>;

    fn fetch_method(&self) -> anyhow::Result<GraphFetchMethod>;

    fn config(&self) -> anyhow::Result<Config>;

    fn config_path(&self) -> Option<&Path>;

    fn hot_reload(&self) -> bool;

    fn log_format<S>(&self) -> BoxedLayer<S>
    where
        S: Subscriber + for<'span> LookupSpan<'span> + Send + Sync;
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
