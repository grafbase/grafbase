use std::{fs, io::ErrorKind, path::PathBuf};

use anyhow::Context;
use clap::Parser;
use federated_server::{Config, GraphFetchMethod};
use grafbase_tracing::otel::layer::BoxedLayer;
use tracing::Subscriber;
use tracing_subscriber::{registry::LookupSpan, Layer};

use super::{log::LogStyle, LogLevel};

#[derive(Debug, Parser)]
#[command(name = "Grafbase Lambda Gateway", version)]
/// Grafbase Lambda Gateway
pub struct Args {
    /// Path to the TOML configuration file
    #[arg(env = "GRAFBASE_CONFIG_PATH", default_value = "./grafbase.toml")]
    pub config: PathBuf,
    /// Path to the schema SDL. If provided, the graph will be static and no connection is made
    /// to the Grafbase API.
    #[arg(env = "GRAFBASE_SCHEMA_PATH", default_value = "./federated.graphql")]
    pub schema: PathBuf,
    /// Set the logging level
    #[arg(env = "GRAFBASE_LOG")]
    pub log_level: Option<LogLevel>,
    /// Set the style of log output
    #[arg(env = "GRAFBASE_LOG_STYLE", default_value_t = LogStyle::Text)]
    log_style: LogStyle,
}

impl Args {
    /// The method of fetching a graph
    pub fn fetch_method(&self) -> anyhow::Result<GraphFetchMethod> {
        let federated_graph = fs::read_to_string(&self.schema).context("could not read federated schema file")?;

        Ok(GraphFetchMethod::FromLocal {
            federated_schema: federated_graph,
        })
    }

    /// The gateway configuration
    pub fn config(&self) -> anyhow::Result<Config> {
        match fs::read_to_string(&self.config) {
            Ok(config) => Ok(toml::from_str(&config)?),
            Err(e) => match e.kind() {
                ErrorKind::NotFound => Ok(Config::default()),
                _ => Err(anyhow::anyhow!("error loading config file: {e}")),
            },
        }
    }

    pub fn log_format<S>(&self) -> BoxedLayer<S>
    where
        S: Subscriber + for<'span> LookupSpan<'span> + Send + Sync,
    {
        let layer = tracing_subscriber::fmt::layer();

        match self.log_style {
            // for interactive terminals we provide colored output
            LogStyle::Text if atty::is(atty::Stream::Stdout) => layer.with_ansi(true).with_target(false).boxed(),
            // for server logs, colors are off
            LogStyle::Text => layer.with_ansi(false).with_target(false).boxed(),
            LogStyle::Json => layer.json().boxed(),
        }
    }
}
