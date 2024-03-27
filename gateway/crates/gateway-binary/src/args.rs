use std::{fs, net::SocketAddr, path::PathBuf};

use anyhow::anyhow;
use ascii::AsciiString;
use clap::{ArgGroup, Parser};
use federated_server::GraphFetchMethod;
use grafbase_tracing::otel::layer::BoxedLayer;
use graph_ref::GraphRef;
use tracing::Subscriber;
use tracing_subscriber::{registry::LookupSpan, Layer};

mod log;

pub(crate) use log::LogLevel;

use self::log::LogStyle;

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
#[command(arg_required_else_help = true)]
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
    #[arg(long, short, env = "GRAFBASE_CONFIG_PATH", default_value = "./grafbase.toml")]
    pub config: PathBuf,
    /// Path to graph SDL. If provided, the graph will be static and no connection is made
    /// to the Grafbase API.
    #[arg(long, short, env = "GRAFBASE_SCHEMA_PATH")]
    pub schema: Option<PathBuf>,
    /// Set the logging level
    #[arg(long = "log", env = "GRAFBASE_LOG")]
    pub log_level: Option<LogLevel>,
    /// Set the style of log output
    #[arg(long, env = "GRAFBASE_LOG_STYLE", default_value_t = LogStyle::Text)]
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
        let layer = tracing_subscriber::fmt::layer();

        match self.log_style {
            // for interactive terminals we provide colored output
            LogStyle::Text if atty::is(atty::Stream::Stdout) => layer.with_ansi(true).boxed(),
            // for server logs, colors are off
            LogStyle::Text => layer.with_ansi(false).boxed(),
            LogStyle::Json => layer.json().boxed(),
        }
    }
}
