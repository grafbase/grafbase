use std::{
    fs,
    net::SocketAddr,
    path::{Path, PathBuf},
};

use anyhow::Context;
use ascii::AsciiString;
use clap::{ArgGroup, Parser};
use federated_server::GraphFetchMethod;
use gateway_config::Config;
use grafbase_telemetry::{
    config::{OtlpExporterConfig, OtlpExporterGrpcConfig, OtlpExporterProtocol},
    otel::layer::BoxedLayer,
};
use graph_ref::GraphRef;
use tracing::Subscriber;
use tracing_subscriber::{registry::LookupSpan, Layer};

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
            .requires("grafbase_access_token")
    )
)]
#[command(name = "Grafbase Gateway", version)]
/// Grafbase Gateway
pub struct Args {
    /// IP address on which the server will listen for incomming connections. Defaults to 127.0.0.1:5000.
    #[arg(short, long)]
    pub listen_address: Option<SocketAddr>,
    #[arg(short, long, help = GraphRef::ARG_DESCRIPTION, env = "GRAFBASE_GRAPH_REF", requires = "grafbase_access_token")]
    pub graph_ref: Option<GraphRef>,
    /// An access token to the Grafbase API. The scope must allow operations on the given account,
    /// and graph defined in the graph-ref argument.
    #[arg(env = "GRAFBASE_ACCESS_TOKEN", hide_env_values(true))]
    pub grafbase_access_token: Option<AsciiString>,
    /// Path to the TOML configuration file
    #[arg(long, short, env = "GRAFBASE_CONFIG_PATH")]
    pub config: Option<PathBuf>,
    /// Path to the schema SDL. If provided, the graph will be static and no connection is made
    /// to the Grafbase API.
    #[arg(long, short, env = "GRAFBASE_SCHEMA_PATH")]
    pub schema: Option<PathBuf>,
    /// Set the logging level
    #[arg(long = "log", env = "GRAFBASE_LOG")]
    pub log_level: Option<LogLevel>,
    /// Set the style of log output
    #[arg(long, env = "GRAFBASE_LOG_STYLE", default_value_t = LogStyle::Text)]
    log_style: LogStyle,
    /// If set, parts of the configuration will get reloaded when changed.
    #[arg(long, action)]
    hot_reload: bool,
}

impl super::Args for Args {
    /// The method of fetching a graph
    fn fetch_method(&self) -> anyhow::Result<GraphFetchMethod> {
        match self.graph_ref.as_ref() {
            Some(graph_ref) => Ok(GraphFetchMethod::FromApi {
                access_token: self
                    .grafbase_access_token
                    .clone()
                    .expect("present due to the arg group"),
                graph_name: graph_ref.graph_slug().to_string(),
                branch: graph_ref.branch_name().map(ToString::to_string),
            }),
            None => {
                let federated_graph =
                    fs::read_to_string(self.schema.as_ref().expect("must exist if graph-ref is not defined"))
                        .context("could not read federated schema file")?;

                Ok(GraphFetchMethod::FromLocal {
                    federated_schema: federated_graph,
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

        if let Some((token, graph_ref)) = self.grafbase_access_token.as_ref().zip(self.graph_ref.as_ref()) {
            config.telemetry.grafbase = Some(OtlpExporterConfig {
                endpoint: "https://otel.grafbase.com".parse().unwrap(),
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
                ..Default::default()
            });
        }

        Ok(config)
    }

    fn log_format<S>(&self) -> BoxedLayer<S>
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

    fn listen_address(&self) -> Option<SocketAddr> {
        self.listen_address
    }

    fn log_level(&self) -> Option<LogLevel> {
        self.log_level
    }
}
