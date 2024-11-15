use std::{
    fs,
    io::IsTerminal,
    net::SocketAddr,
    path::{Path, PathBuf},
};

use anyhow::Context;
use ascii::AsciiString;
use clap::{ArgGroup, Parser};
use federated_server::GraphFetchMethod;
use gateway_config::{BatchExportConfig, Config};
use grafbase_telemetry::config::{OtlpExporterConfig, OtlpExporterGrpcConfig, OtlpExporterProtocol};
use graph_ref::GraphRef;

use super::{log::LogStyle, LogLevel};

#[derive(Debug, Parser)]
#[clap(
    group(
        ArgGroup::new("hybrid-or-airgapped")
            .required(true)
            .args(["graph_ref", "schema"])
    ),
    group(
        ArgGroup::new("graph-ref-with-access-token")
            .args(["graph_ref"])
    )
)]
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

impl Args {
    pub fn grafbase_access_token(&self) -> Option<String> {
        std::env::var("GRAFBASE_ACCESS_TOKEN").ok()
    }
}

impl super::Args for Args {
    /// The method of fetching a graph
    fn fetch_method(&self) -> anyhow::Result<GraphFetchMethod> {
        match self.graph_ref.clone() {
            Some(graph_ref) => Ok(GraphFetchMethod::FromGraphRef {
                access_token: AsciiString::from_ascii(self.grafbase_access_token().ok_or_else(|| {
                    anyhow::format_err!(
                        "The GRAFBASE_ACCESS_TOKEN environment variable must be set when a graph_ref is provided"
                    )
                })?)?,
                graph_ref,
            }),
            None => {
                let federated_sdl =
                    fs::read_to_string(self.schema.as_ref().expect("must exist if graph-ref is not defined"))
                        .context("could not read federated schema file")?;

                Ok(GraphFetchMethod::FromSchema {
                    federated_sdl,
                    reload_signal: None,
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
        let mut config = match self.config.as_ref() {
            Some(path) => {
                let config = fs::read_to_string(path).context("could not read config file")?;
                toml::from_str(&config)?
            }
            None => Config::default(),
        };

        if let Some((token, graph_ref)) = self.grafbase_access_token().as_ref().zip(self.graph_ref.as_ref()) {
            config.telemetry.grafbase = Some(OtlpExporterConfig {
                endpoint: std::env::var("__GRAFBASE_OTEL_URL")
                    .unwrap_or("https://otel.grafbase.com".to_string())
                    .parse()
                    .unwrap(),
                enabled: true,
                protocol: OtlpExporterProtocol::Grpc,
                grpc: Some(OtlpExporterGrpcConfig {
                    tls: None,
                    headers: vec![
                        (
                            AsciiString::from_ascii(b"authorization").context("Invalid auth header name")?,
                            AsciiString::from_ascii(format!("Bearer {token}")).context("Invalid access token")?,
                        ),
                        (
                            AsciiString::from_ascii(b"grafbase-graph-ref").context("Invalid graph ref header name")?,
                            AsciiString::from_ascii(graph_ref.to_string().into_bytes()).context("Invalid graph ref")?,
                        ),
                    ]
                    .into(),
                }),
                batch_export: if let Some(seconds) = std::env::var("__GRAFBASE_OTEL_EXPORT_DELAY")
                    .ok()
                    .and_then(|v| v.parse::<usize>().ok())
                {
                    BatchExportConfig {
                        scheduled_delay: chrono::Duration::seconds(seconds as i64),
                        ..Default::default()
                    }
                } else {
                    Default::default()
                },
                ..Default::default()
            });
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
