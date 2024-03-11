use super::{
    gateway::{self, GatewayWatcher},
    graph_updater::GraphUpdater,
};
use crate::config::{AuthenticationConfig, OperationLimitsConfig};
use ascii::AsciiString;
use std::sync::Arc;
use tokio::sync::watch;
use tracing::Level;

/// The method of running the gateway.
pub enum GraphFetchMethod {
    /// The schema is fetched in regular intervals from the Grafbase API.
    FromApi {
        /// The access token for accessing the the API.
        access_token: AsciiString,
        /// The name of the graph
        graph_name: String,
        /// The graph branch
        branch: Option<String>,
    },
    /// The schema is loaded from disk. No access to the Grafbase API.
    FromLocal {
        /// Static federated graph from a file
        federated_schema: String,
    },
}

impl GraphFetchMethod {
    /// Converts the fetch method into an eventually existing gateway. This can happen
    /// in two ways: if providing a graph SDL, we a new gateway immediately. Alternatively,
    /// if a graph ref and access token is provided, the function returns immediately, and
    /// the gateway will be available eventually when the GDN responds with a working graph.
    pub(crate) fn into_gateway(
        self,
        enable_introspection: bool,
        operation_limits: Option<OperationLimitsConfig>,
        authentication: Option<AuthenticationConfig>,
    ) -> crate::Result<GatewayWatcher> {
        let (sender, gateway) = watch::channel(None);

        match self {
            GraphFetchMethod::FromApi {
                access_token,
                graph_name,
                branch,
            } => {
                tokio::spawn(async move {
                    let mut updater = GraphUpdater::new(&graph_name, branch.as_deref(), access_token, sender)?
                        .enable_introspection(enable_introspection);

                    if let Some(operation_limits) = operation_limits {
                        updater = updater.with_operation_limits(operation_limits);
                    }

                    if let Some(auth_config) = authentication {
                        updater = updater.with_authentication(auth_config);
                    }

                    updater.poll().await;

                    Ok::<_, crate::Error>(())
                });
            }
            GraphFetchMethod::FromLocal { federated_schema } => {
                tracing::event!(
                    Level::INFO,
                    message = "creating a new gateway",
                    operation_limits = operation_limits.is_some(),
                    introspection_enabled = enable_introspection,
                    authentication = authentication.is_some(),
                );

                let gateway = gateway::generate(
                    &federated_schema,
                    operation_limits,
                    authentication,
                    enable_introspection,
                )?;

                sender.send(Some(Arc::new(gateway)))?;
            }
        }

        Ok(gateway)
    }
}
