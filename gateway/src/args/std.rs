use std::{
    fs,
    io::IsTerminal,
    net::SocketAddr,
    path::{Path, PathBuf},
};

use anyhow::Context;
use clap::Parser;
use federated_server::GraphFetchMethod;
use gateway_config::Config;
use graph_ref::GraphRef;

use super::{LogLevel, log::LogStyle};

#[derive(Debug, Parser)]
#[command(name = "Grafbase Gateway", version)]
/// Grafbase Gateway
pub struct Args {
    /// IP address on which the server will listen for incomming connections. Defaults to 127.0.0.1:5000.
    #[arg(short, long)]
    pub listen_address: Option<SocketAddr>,
    #[arg(short, long, help = GraphRef::ARG_DESCRIPTION, env = "GRAFBASE_GRAPH_REF")]
    pub graph_ref: Option<GraphRef>,
    /// Path to the TOML configuration file
    #[arg(long, short, env = "GRAFBASE_CONFIG_PATH")]
    pub config: Option<PathBuf>,
    /// Path to the schema SDL. If provided, the graph will be static and no connection is made
    /// to the Grafbase API.
    #[arg(long, short, env = "GRAFBASE_SCHEMA_PATH")]
    pub schema: Option<PathBuf>,
    /// Set the logging level, this applies to all spans, logs and trace events.
    ///
    /// Beware that *only* 'off', 'error', 'warn' and 'info' can be used safely in production. More
    /// verbose levels, such as 'debug', will include sensitive information like request variables, responses, etc.
    ///
    /// Possible values are: 'off', 'error', 'warn', 'info', 'debug', 'trace' or a custom string.
    /// In the last case, the string is passed on to [`tracing_subscriber::EnvFilter`] as is and is
    /// only meant for debugging purposes. No stability guarantee is made on the format.
    #[arg(long = "log", env = "GRAFBASE_LOG", default_value = "info")]
    pub log_level: String,
    /// Set the style of log output
    #[arg(long, env = "GRAFBASE_LOG_STYLE")]
    log_style: Option<LogStyle>,
    /// If set, parts of the configuration will get reloaded when changed.
    #[arg(long, action)]
    hot_reload: bool,
}

impl super::Args for Args {
    fn graph_ref(&self) -> Option<&GraphRef> {
        self.graph_ref.as_ref()
    }

    /// The method of fetching a graph
    fn fetch_method(&self) -> anyhow::Result<GraphFetchMethod> {
        match self.schema {
            Some(ref schema) => {
                let schema_path = schema.to_owned();
                let federated_sdl = fs::read_to_string(&schema_path).context("could not read federated schema file")?;

                Ok(GraphFetchMethod::FromSchema {
                    federated_sdl,
                    schema_path,
                })
            }
            None => {
                let graph_ref = self.graph_ref.clone().ok_or_else(|| {
                    anyhow::format_err!("The graph-ref argument must be set if not using a static schema file.")
                })?;

                let access_token = self.grafbase_access_token()?.ok_or_else(|| {
                    anyhow::format_err!(
                        "The GRAFBASE_ACCESS_TOKEN environment variable must be set when a graph-ref is provided"
                    )
                })?;

                Ok(GraphFetchMethod::FromGraphRef {
                    access_token,
                    graph_ref,
                })
            }
        }
    }

    fn config_path(&self) -> Option<&Path> {
        self.config.as_deref()
    }

    fn hot_reload(&self) -> bool {
        self.hot_reload
    }

    fn config(&self) -> anyhow::Result<Config> {
        let mut config = Config::loader()
            .load(self.config.as_deref())
            .map_err(|err| anyhow::anyhow!(err))?
            .unwrap_or_default();

        if let Some(otel_config) = self.grafbase_otel_config()? {
            config.telemetry.grafbase = Some(otel_config);
        }

        Ok(config)
    }

    fn log_style(&self) -> LogStyle {
        self.log_style.unwrap_or_else(|| {
            let log_level = self.log_level();
            if std::io::stdout().is_terminal() && (log_level.contains("debug") || log_level.contains("trace")) {
                LogStyle::Pretty
            } else {
                LogStyle::Text
            }
        })
    }

    fn listen_address(&self) -> Option<SocketAddr> {
        self.listen_address
    }

    fn log_level(&self) -> LogLevel<'_> {
        LogLevel(&self.log_level)
    }
}
