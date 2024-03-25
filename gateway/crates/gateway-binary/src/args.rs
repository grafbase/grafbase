use std::{fs, net::SocketAddr, path::PathBuf};

use anyhow::anyhow;
use ascii::AsciiString;
use clap::{ArgGroup, Parser};
use federated_server::GraphFetchMethod;
use graph_ref::GraphRef;

/// the tracing filter to be used when tracing is on
const TRACE_LOG_FILTER: &str = "tower_http=debug,federated_dev=trace,engine_v2=debug,federated_server=trace";
/// the tracing filter to be used when tracing is off
const DEFAULT_LOG_FILTER: &str = "info";

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
    ),
    group(
        ArgGroup::new("schema-with-license")
            .args(["schema"])
            .requires("license")
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
    /// to the Grafbase API. A license must be present if defined.
    #[arg(long, short, env = "GRAFBASE_SCHEMA_PATH")]
    pub schema: Option<PathBuf>,
    /// Path to a Grafbase license file. Must be provided if defining a schema path.
    #[arg(long, env = "GRAFBASE_LICENSE_PATH")]
    pub license: Option<PathBuf>,
    /// Set the tracing level
    #[arg(short, long, default_value_t = 0)]
    pub trace: u16,
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

    /// Defines the log level for associated crates
    pub fn log_filter(&self) -> &str {
        if self.trace >= 1 {
            TRACE_LOG_FILTER
        } else {
            DEFAULT_LOG_FILTER
        }
    }
}
