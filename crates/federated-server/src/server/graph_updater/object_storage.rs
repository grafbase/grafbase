use crate::{
    AccessToken, ObjectStorageResponse,
    server::{events::UpdateEvent, gateway::GraphDefinition},
};

use grafbase_telemetry::{
    metrics::meter_from_global_provider,
    otel::opentelemetry::{KeyValue, metrics::Histogram},
};
use graph_ref::GraphRef;
use http::{HeaderValue, StatusCode};
use std::{
    borrow::Cow,
    time::{Duration, SystemTime},
};
use tokio::sync::mpsc;
use tokio::time::MissedTickBehavior;
use ulid::Ulid;
use url::Url;

/// How often we poll updates to the graph.
const TICK_INTERVAL: Duration = Duration::from_secs(10);

/// How long we wait for a response from object storage.
const OBJECT_STORAGE_TIMEOUT: Duration = Duration::from_secs(10);

/// How long we wait until a connection is successfully opened.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

/// Sets an interval for HTTP2 Ping frames should be sent to keep a connection alive.
const KEEPALIVE_INTERVAL: Duration = Duration::from_secs(1);

/// Sets a timeout for receiving an acknowledgement of the keep-alive ping.
const KEEPALIVE_TIMEOUT: Duration = Duration::from_secs(5);

/// Sets whether HTTP2 keep-alive should apply while the connection is idle.
const KEEPALIVE_WHILE_IDLE: bool = true;

/// The HTTP user-agent header we sent to the schema registry.
const USER_AGENT: &str = "grafbase-cli";

/// The default CDN host we load the graphs from.
const DEFAULT_OBJECT_STORAGE_HOST: &str = "https://object-storage.grafbase.com";

/// The name of the environment variable used to read the object storage host from.
const OBJECT_STORAGE_HOST_ENV_VAR: &str = "GRAFBASE_OBJECT_STORAGE_URL";

#[derive(Debug, Clone, Copy)]
enum ResponseKind {
    /// Indicates that a new graph version has been fetched.
    New,
    /// Indicates that there are no changes to the current graph version.
    Unchanged,
    /// Indicates that an HTTP error occurred while fetching the graph.
    HttpError,
    /// Indicates that a Grafbase-specific error occurred.
    ObjectStorageError,
}

impl ResponseKind {
    fn as_str(self) -> &'static str {
        match self {
            ResponseKind::New => "NEW",
            ResponseKind::Unchanged => "UNCHANGED",
            ResponseKind::HttpError => "HTTP_ERROR",
            ResponseKind::ObjectStorageError => "OBJECT_STORAGE_ERROR",
        }
    }
}

struct ObjectStorageFetchLatencyAttributes {
    /// The kind of response received from the server.
    kind: ResponseKind,
    /// The HTTP status code of the response, if applicable.
    status_code: Option<StatusCode>,
}

/// A struct representing a GraphUpdater, which is responsible for polling updates
/// from object storage and managing the associated state.
pub struct ObjectStorageUpdater {
    object_storage_url: Url,
    object_storage_client: reqwest::Client,
    access_token: AccessToken,
    sender: mpsc::Sender<UpdateEvent>,
    current_id: Option<Ulid>,
    latencies: Histogram<u64>,
}

impl ObjectStorageUpdater {
    /// Creates a new instance of `GraphUpdater`.
    ///
    /// # Arguments
    ///
    /// * `graph_ref` - A reference to the graph to be updated.
    /// * `access_token` - The access token for authentication with the object storage service.
    /// * `sender` - The sender used to send a new instance of the gateway to the server.
    /// * `gateway_config` - Configuration settings for the gateway.
    /// * `hooks` - Hooks for custom behavior during operation execution.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be built or if the URL parsing fails.
    pub fn new(
        graph_ref: GraphRef,
        access_token: AccessToken,
        sender: mpsc::Sender<UpdateEvent>,
    ) -> crate::Result<Self> {
        let object_storage_client = reqwest::ClientBuilder::new()
            .timeout(OBJECT_STORAGE_TIMEOUT)
            .connect_timeout(CONNECT_TIMEOUT)
            .http2_keep_alive_interval(Some(KEEPALIVE_INTERVAL))
            .http2_keep_alive_timeout(KEEPALIVE_TIMEOUT)
            .http2_keep_alive_while_idle(KEEPALIVE_WHILE_IDLE)
            .user_agent(USER_AGENT)
            .build()
            .map_err(|e| crate::Error::InternalError(e.to_string()))?;

        let object_storage_host = match std::env::var(OBJECT_STORAGE_HOST_ENV_VAR) {
            Ok(host) => Cow::Owned(host),
            Err(_) => Cow::Borrowed(DEFAULT_OBJECT_STORAGE_HOST),
        };

        let object_storage_url = match graph_ref {
            GraphRef::LatestProductionVersion { graph_slug } => {
                format!("{object_storage_host}/graphs/{graph_slug}")
            }
            GraphRef::LatestVersion {
                graph_slug,
                branch_name,
            } => format!("{object_storage_host}/graphs/{graph_slug}/branch/{branch_name}"),
            GraphRef::Id {
                graph_slug,
                branch_name,
                version,
            } => format!("{object_storage_host}/graphs/{graph_slug}/branch/{branch_name}/version/{version}"),
        };

        let object_storage_url = object_storage_url
            .parse::<Url>()
            .map_err(|e| crate::Error::InternalError(e.to_string()))?;

        Ok(Self {
            object_storage_url,
            object_storage_client,
            access_token,
            sender,
            current_id: None,
            latencies: meter_from_global_provider()
                .u64_histogram("object_storage.request.duration")
                .build(),
        })
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

        // if we have a slow connection, this prevents bursts of connections to object storage
        // for all the missed ticks.
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {
            interval.tick().await;

            let mut request = self
                .object_storage_client
                .get(self.object_storage_url.as_str())
                .bearer_auth(&self.access_token);

            if let Some(id) = self.current_id {
                request = request.header(
                    "If-None-Match",
                    HeaderValue::from_bytes(id.to_string().as_bytes()).expect("must be ascii"),
                );
            }

            let start = SystemTime::now();
            let response = request.send().await;
            let duration = SystemTime::now().duration_since(start).unwrap_or_default();

            let response = match response {
                Ok(response) => response,
                Err(e) => {
                    self.record_duration(
                        ObjectStorageFetchLatencyAttributes {
                            kind: ResponseKind::HttpError,
                            status_code: None,
                        },
                        duration,
                    );

                    tracing::error!("Failed to update graph: {e}");
                    continue;
                }
            };

            if response.status() == StatusCode::NOT_MODIFIED {
                self.record_duration(
                    ObjectStorageFetchLatencyAttributes {
                        kind: ResponseKind::Unchanged,
                        status_code: Some(response.status()),
                    },
                    duration,
                );

                tracing::trace!("no updates to the graph");
                continue;
            }

            if let Err(e) = response.error_for_status_ref() {
                self.record_duration(
                    ObjectStorageFetchLatencyAttributes {
                        kind: ResponseKind::ObjectStorageError,
                        status_code: e.status(),
                    },
                    duration,
                );

                match e.status() {
                    Some(StatusCode::NOT_FOUND) => {
                        tracing::warn!(
                            "Federated schema not found. Is your graph configured as self-hosted? Did you publish at least one subgraph?"
                        );
                    }
                    _ => {
                        tracing::error!("Failed to update graph: {e}");
                    }
                }
                continue;
            }

            let response: ObjectStorageResponse = match response.json().await {
                Ok(response) => response,
                Err(e) => {
                    self.record_duration(
                        ObjectStorageFetchLatencyAttributes {
                            kind: ResponseKind::ObjectStorageError,
                            status_code: e.status(),
                        },
                        duration,
                    );

                    tracing::error!("Failed to update graph: {e}");
                    continue;
                }
            };

            tracing::info!("Finished fetching new graph");

            let version_id = response.version_id;

            self.record_duration(
                ObjectStorageFetchLatencyAttributes {
                    kind: ResponseKind::New,
                    status_code: None,
                },
                duration,
            );

            self.current_id = Some(version_id);

            self.sender
                .send(UpdateEvent::Graph(GraphDefinition::ObjectStorage {
                    response,
                    object_storage_base_url: self.object_storage_url.clone(),
                }))
                .await
                .expect("internal error: channel closed");
        }
    }

    fn record_duration(
        &self,
        ObjectStorageFetchLatencyAttributes { kind, status_code }: ObjectStorageFetchLatencyAttributes,
        duration: Duration,
    ) {
        let mut attributes = vec![
            KeyValue::new("server.address", self.object_storage_url.to_string()),
            KeyValue::new("object_storage.response.kind", kind.as_str()),
        ];

        if let Some(status_code) = status_code {
            attributes.push(KeyValue::new(
                "http.response.status.code",
                status_code.as_u16().to_string(),
            ));
        }

        self.latencies.record(duration.as_millis() as u64, &attributes);
    }
}
