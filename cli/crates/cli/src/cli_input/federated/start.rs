use std::{fs, net::SocketAddr, path::PathBuf};

use ascii::AsciiString;
use clap::ArgGroup;
use production_server::GraphFetchMethod;

use crate::{cli_input::GraphRef, errors::CliError};

#[derive(Debug, clap::Args)]
#[clap(
    group(
        ArgGroup::new("hybrid-or-airgapped")
            .required(true)
            .args(["graph_ref", "federated_schema"])
    ),
    group(
        ArgGroup::new("graph-ref-with-access-token")
            .args(["graph_ref"])
            .requires("access_token")
    )
)]
pub struct FederatedStartCommand {
    /// IP address on which the server will listen for incomming connections. Defaults to 127.0.0.1:4000.
    #[arg(long)]
    pub listen_address: Option<SocketAddr>,
    #[arg(long, help = GraphRef::ARG_DESCRIPTION, env = "GRAFBASE_GRAPH_REF")]
    pub graph_ref: Option<GraphRef>,
    /// An access token to the Grafbase API. The scope must allow operations on the given account,
    /// and graph defined in the graph-ref argument.
    #[arg(long, env = "GRAFBASE_ACCESS_TOKEN")]
    pub access_token: Option<AsciiString>,
    /// Path to the TOML configuration file
    #[arg(long)]
    pub config: PathBuf,
    /// Path to federated graph SDL. If provided, the graph will be static and no connection is made
    /// to the Grafbase API.
    #[arg(long)]
    pub federated_schema: Option<PathBuf>,
}

impl FederatedStartCommand {
    pub fn fetch_method(&self) -> Result<GraphFetchMethod, CliError> {
        match (self.graph_ref.as_ref(), self.federated_schema.as_ref()) {
            (None, Some(path)) => {
                let federated_graph = fs::read_to_string(path)
                    .map_err(|e| CliError::InvalidArgumentsError(format!("error loading federated schema:\n{e}")))?;

                Ok(GraphFetchMethod::FromLocal {
                    federated_schema: federated_graph,
                })
            }
            (Some(graph_ref), None) => Ok(GraphFetchMethod::FromApi {
                access_token: self.access_token.clone().expect("present due to the arg group"),
                graph_name: graph_ref.graph().to_string(),
                branch: graph_ref.branch().map(ToString::to_string),
            }),
            _ => unreachable!(),
        }
    }
}
