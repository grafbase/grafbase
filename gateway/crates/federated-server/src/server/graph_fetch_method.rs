use super::gateway::{self, GatewayWatcher};
use crate::server::gateway::GatewayConfig;
use licensing::{JWTClaims, License};
use std::sync::Arc;
use tokio::sync::watch;
use tracing::Level;

/// The method of running the gateway.
pub enum GraphFetchMethod {
    /// The schema is fetched in regular intervals from the Grafbase API.
    #[cfg(not(feature = "lambda"))]
    FromApi {
        /// The access token for accessing the the API.
        access_token: ascii::AsciiString,
        /// The name of the graph
        graph_name: String,
        /// The graph branch
        branch: Option<String>,
    },
    /// The schema is loaded from disk. No access to the Grafbase API.
    FromLocal {
        /// Static federated graph from a file
        federated_schema: String,
        /// A valid license for the gateway
        license: Option<JWTClaims<License>>,
    },
}

impl GraphFetchMethod {
    /// Converts the fetch method into an eventually existing gateway. This can happen
    /// in two ways: if providing a graph SDL, we a new gateway immediately. Alternatively,
    /// if a graph ref and access token is provided, the function returns immediately, and
    /// the gateway will be available eventually when the GDN responds with a working graph.
    pub(crate) fn into_gateway(self, config: GatewayConfig) -> crate::Result<GatewayWatcher> {
        let (sender, gateway) = watch::channel(None);

        match self {
            #[cfg(not(feature = "lambda"))]
            GraphFetchMethod::FromApi {
                access_token,
                graph_name,
                branch,
            } => {
                tokio::spawn(async move {
                    use super::graph_updater::GraphUpdater;

                    GraphUpdater::new(&graph_name, branch.as_deref(), access_token, sender, config)?
                        .poll()
                        .await;

                    Ok::<_, crate::Error>(())
                });
            }
            GraphFetchMethod::FromLocal {
                federated_schema,
                license,
            } => {
                if let Some(license) = license.as_ref() {
                    if licensing::in_grace_period(license) {
                        let days_left = license.expires_at.expect("must be some").as_days();

                        tracing::event!(
                            Level::WARN,
                            message = "the provided license is expiring soon, please acquire a new license to continue using the Grafbase Gateway",
                            days_left = days_left,
                        );
                    }
                }

                tracing::event!(
                    Level::INFO,
                    message = "creating a new gateway",
                    operation_limits = config.operation_limits.is_some(),
                    introspection_enabled = config.enable_introspection,
                    authentication = config.authentication.is_some(),
                );

                let gateway = gateway::generate(&federated_schema, None, config)?;

                sender.send(Some(Arc::new(gateway)))?;
            }
        }

        Ok(gateway)
    }
}
