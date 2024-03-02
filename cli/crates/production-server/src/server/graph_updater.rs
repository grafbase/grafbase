use std::{borrow::Cow, sync::Arc, time::Duration};

use super::gateway::EngineSender;
use crate::config::{AuthenticationConfig, OperationLimitsConfig};
use ascii::AsciiString;
use http::{HeaderValue, StatusCode};
use tracing::Level;
use ulid::Ulid;
use url::Url;

/// How often we poll updates from the schema registry.
const TICK_INTERVAL: Duration = Duration::from_secs(10);

/// How long we wait for a response from the schema registry.
const UPLINK_TIMEOUT: Duration = Duration::from_secs(30);

/// How long we keep the HTTP connection alive in the pool.
const KEEPALIVE_DURATION: Duration = Duration::from_secs(60);

/// The HTTP user-agent header we sent to the schema registry.
const USER_AGENT: &str = "grafbase-cli";

/// The CDN host we load the graphs from.
const UPLINK_HOST: &str = "https://gdn.grafbase.com";

/// An updater thread for polling graph changes from the API.
pub(super) struct GraphUpdater {
    graph_ref: String,
    uplink_url: Url,
    uplink_client: reqwest::Client,
    access_token: AsciiString,
    sender: EngineSender,
    current_id: Option<Ulid>,
    operation_limits_config: Option<OperationLimitsConfig>,
    authentication_config: Option<AuthenticationConfig>,
    enable_introspection: bool,
}

/// TODO: here you get the needed values for tracing Hugo!
#[derive(Debug, serde::Deserialize)]
#[allow(dead_code)]
struct UplinkResponse {
    account_id: Ulid,
    graph_id: Ulid,
    branch: String,
    sdl: String,
    version_id: Ulid,
}

impl GraphUpdater {
    pub fn new(
        graph_ref: &str,
        branch: Option<&str>,
        access_token: AsciiString,
        sender: EngineSender,
    ) -> crate::Result<Self> {
        let uplink_client = reqwest::ClientBuilder::new()
            .gzip(true)
            .timeout(UPLINK_TIMEOUT)
            .connect_timeout(Duration::from_secs(5))
            .tcp_keepalive(KEEPALIVE_DURATION)
            .user_agent(USER_AGENT)
            .build()
            .map_err(|e| crate::Error::InternalError(e.to_string()))?;

        let uplink_host = match std::env::var("GRAFBASE_UPLINK_HOST") {
            Ok(host) => Cow::Owned(host),
            Err(_) => Cow::Borrowed(UPLINK_HOST),
        };

        let uplink_url = match branch {
            Some(branch) => format!("{uplink_host}/graphs/{graph_ref}/{branch}/current"),
            None => format!("{uplink_host}/graphs/{graph_ref}/current"),
        };

        let uplink_url = uplink_url
            .parse::<Url>()
            .map_err(|e| crate::Error::InternalError(e.to_string()))?;

        Ok(Self {
            graph_ref: graph_ref.to_string(),
            uplink_url,
            uplink_client,
            access_token,
            sender,
            current_id: None,
            operation_limits_config: None,
            authentication_config: None,
            enable_introspection: false,
        })
    }

    /// Adds operation limits to the updated graph from the configuration.
    pub fn with_operation_limits(mut self, config: OperationLimitsConfig) -> Self {
        self.operation_limits_config = Some(config);
        self
    }

    /// Sets the JWT authentication for the server.
    pub fn with_authentication(mut self, config: AuthenticationConfig) -> Self {
        self.authentication_config = Some(config);
        self
    }

    /// Enables introspection to the updated graphs.
    pub fn enable_introspection(mut self, value: bool) -> Self {
        self.enable_introspection = value;
        self
    }

    /// A poll loop for fetching the latest graph from the API. When started,
    /// fetches the graph immediately and after that every ten seconds. If we detect
    /// changes to the running graph, we create a new gateway and replace the running
    /// one with it.
    ///
    /// By having the gateway in a reference counter, we make sure the current requests
    /// are served before dropping.
    pub async fn poll(&mut self) {
        let mut interval = tokio::time::interval(TICK_INTERVAL);

        loop {
            interval.tick().await;

            let mut request = self
                .uplink_client
                .get(self.uplink_url.as_str())
                .bearer_auth(&self.access_token);

            if let Some(id) = self.current_id {
                request = request.header(
                    "If-None-Match",
                    HeaderValue::from_bytes(id.to_string().as_bytes()).expect("must be ascii"),
                );
            }

            let response = request.send().await;

            let response = match response {
                Ok(response) => response,
                Err(e) => {
                    tracing::event!(Level::ERROR, message = "error updating graph", error = e.to_string());
                    continue;
                }
            };

            if response.status() == StatusCode::NOT_MODIFIED {
                tracing::debug!("no updates to the graph");
                continue;
            }

            if let Err(e) = response.error_for_status_ref() {
                match e.status() {
                    Some(StatusCode::NOT_FOUND) => {
                        tracing::warn!("no subgraphs registered, publish at least one subgraph");
                    }
                    _ => {
                        tracing::event!(Level::ERROR, message = "error updating graph", error = e.to_string());
                    }
                }
                continue;
            }

            let response: UplinkResponse = match response.json().await {
                Ok(response) => response,
                Err(e) => {
                    tracing::event!(Level::ERROR, message = "error updating graph", error = e.to_string());
                    continue;
                }
            };

            let gateway = match super::gateway::generate(
                &response.sdl,
                self.operation_limits_config,
                self.authentication_config.clone(),
                self.enable_introspection,
            ) {
                Ok(gateway) => gateway,
                Err(e) => {
                    tracing::event!(Level::ERROR, message = "error parsing graph", error = e.to_string());
                    continue;
                }
            };

            tracing::event!(
                Level::INFO,
                message = "creating a new gateway",
                graph_ref = self.graph_ref,
                branch = response.branch,
                operation_limits = self.operation_limits_config.is_some(),
                introspection_enabled = self.enable_introspection,
                authentication = self.authentication_config.is_some(),
            );

            self.current_id = Some(response.version_id);

            self.sender
                .send(Some(Arc::new(gateway)))
                .expect("internal error: channel closed");
        }
    }
}
