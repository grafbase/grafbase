use std::{fmt, fs, net::SocketAddr, path::PathBuf};

use anyhow::anyhow;
use ascii::AsciiString;
use clap::{ArgGroup, Parser, ValueEnum};
use federated_server::GraphFetchMethod;
use grafbase_tracing::otel::layer::BoxedLayer;
use graph_ref::GraphRef;
use tracing::Subscriber;
use tracing_subscriber::{registry::LookupSpan, Layer};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub(crate) enum LogLevel {
    /// Completely disables logging
    Off,
    /// Only errors from Grafbase libraries
    Error,
    /// Warnings and errors from Grafbase libraries
    Warn,
    /// Info, warning and error messages from Grafbase libraries
    Info,
    /// Debug, info, warning and error messages from all dependencies
    Debug,
    /// Trace, debug, info, warning and error messages from all dependencies
    Trace,
}

impl Default for LogLevel {
    fn default() -> Self {
        Self::Info
    }
}

impl LogLevel {
    pub(crate) fn as_filter_str(&self) -> &'static str {
        match self {
            LogLevel::Off => "off",
            LogLevel::Error => "grafbase=error,off",
            LogLevel::Warn => "grafbase=warn,off",
            LogLevel::Info => "grafbase=info,off",
            LogLevel::Debug => "debug",
            LogLevel::Trace => "trace",
        }
    }
}

impl AsRef<str> for LogLevel {
    fn as_ref(&self) -> &str {
        match self {
            LogLevel::Off => "off",
            LogLevel::Error => "error",
            LogLevel::Warn => "warn",
            LogLevel::Info => "info",
            LogLevel::Debug => "debug",
            LogLevel::Trace => "trace",
        }
    }
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_ref())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum LogStyle {
    /// Standard text
    Ascii,
    /// Standard text with ANSI coloring
    Ansi,
    /// JSON objects
    Json,
}

impl AsRef<str> for LogStyle {
    fn as_ref(&self) -> &str {
        match self {
            LogStyle::Ascii => "ascii",
            LogStyle::Ansi => "ansi",
            LogStyle::Json => "json",
        }
    }
}

impl fmt::Display for LogStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_ref())
    }
}

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
#[command(name = "The Grafbase Gateway", version)]
/// The Grafbase Gateway
pub struct Args {
    /// IP address on which the server will listen for incomming connections. Defaults to 127.0.0.1:5000.
    #[arg(short, long)]
    pub listen_address: Option<SocketAddr>,
    #[arg(short, long, help = GraphRef::ARG_DESCRIPTION, env = "GRAFBASE_GRAPH_REF")]
    pub graph_ref: Option<GraphRef>,
    /// An access token to the Grafbase API. The scope must allow operations on the given account,
    /// and graph defined in the graph-ref argument.
    #[arg(env = "GRAFBASE_ACCESS_TOKEN")]
    pub grafbase_access_token: Option<AsciiString>,
    /// Path to the TOML configuration file
    #[arg(long, short, env = "GRAFBASE_CONFIG_PATH")]
    pub config: PathBuf,
    /// Path to graph SDL. If provided, the graph will be static and no connection is made
    /// to the Grafbase API.
    #[arg(long, short, env = "GRAFBASE_SCHEMA_PATH")]
    pub schema: Option<PathBuf>,
    /// Set the tracing and logging level
    #[arg(long = "log", env = "GRAFBASE_LOG")]
    pub log_level: Option<LogLevel>,
    /// Set the style of log output
    #[arg(long, env = "GRAFBASE_LOG_STYLE", default_value_t = LogStyle::Ascii)]
    log_style: LogStyle,
}

impl Args {
    /// The method of fetching a graph
    pub fn fetch_method(&self) -> anyhow::Result<GraphFetchMethod> {
        match (self.graph_ref.as_ref(), self.schema.as_ref()) {
            (None, Some(path)) => {
                let federated_graph = fs::read_to_string(path).map_err(|e| anyhow!("error loading schema:\n{e}"))?;

                Ok(GraphFetchMethod::FromLocal {
                    federated_schema: federated_graph,
                })
            }
            #[cfg(not(feature = "lambda"))]
            (Some(graph_ref), None) => Ok(GraphFetchMethod::FromApi {
                access_token: self
                    .grafbase_access_token
                    .clone()
                    .expect("present due to the arg group"),
                graph_name: graph_ref.graph().to_string(),
                branch: graph_ref.branch().map(ToString::to_string),
            }),
            #[cfg(feature = "lambda")]
            (Some(_), None) => {
                let error = anyhow!("Hybrid mode is not available for lambda deployments, please provide the full GraphQL schema as a file.");

                Err(error)
            }
            _ => unreachable!(),
        }
    }

    pub fn log_format<S>(&self) -> BoxedLayer<S>
    where
        S: Subscriber + for<'span> LookupSpan<'span> + Send + Sync,
    {
        match self.log_style {
            LogStyle::Ascii => tracing_subscriber::fmt::layer().with_ansi(false).boxed(),
            LogStyle::Ansi => tracing_subscriber::fmt::layer().with_ansi(true).boxed(),
            LogStyle::Json => tracing_subscriber::fmt::layer().json().boxed(),
        }
    }
}
