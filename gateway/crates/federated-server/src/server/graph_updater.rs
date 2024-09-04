use std::time::SystemTime;
use std::{borrow::Cow, sync::Arc, time::Duration};

use crate::server::gateway::GraphDefinition;

use super::gateway::GatewaySender;
use ascii::AsciiString;
use gateway_config::Config;
use grafbase_telemetry::metrics::meter_from_global_provider;
use grafbase_telemetry::otel::opentelemetry::metrics::Histogram;
use grafbase_telemetry::otel::opentelemetry::KeyValue;
use graph_ref::GraphRef;
use http::{HeaderValue, StatusCode};
use runtime_local::hooks::ChannelLogSender;
use tokio::time::MissedTickBehavior;
use ulid::Ulid;
use url::Url;

use super::GdnResponse;

/// How often we poll updates from the schema registry.
const TICK_INTERVAL: Duration = Duration::from_secs(10);

/// How long we wait for a response from the schema registry.
const GDN_TIMEOUT: Duration = Duration::from_secs(10);

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

/// The CDN host we load the graphs from.
const GDN_HOST: &str = "https://gdn.grafbase.com";

#[derive(Debug, Clone, Copy)]
enum ResponseKind {
    New,
    Unchanged,
    HttpError,
    GdnError,
}

impl ResponseKind {
    fn as_str(self) -> &'static str {
        match self {
            ResponseKind::New => "NEW",
            ResponseKind::Unchanged => "UNCHANGED",
            ResponseKind::HttpError => "HTTP_ERROR",
            ResponseKind::GdnError => "GDN_ERROR",
        }
    }
}

struct GdnFetchLatencyAttributes {
    kind: ResponseKind,
    status_code: Option<StatusCode>,
}

/// An updater thread for polling graph changes from the API.
pub(super) struct GraphUpdater {
    gdn_url: Url,
    gdn_client: reqwest::Client,
    access_token: AsciiString,
    sender: GatewaySender,
    current_id: Option<Ulid>,
    gateway_config: Config,
    latencies: Histogram<u64>,
    access_log_sender: ChannelLogSender,
}

impl GraphUpdater {
    pub fn new(
        graph_ref: GraphRef,
        access_token: AsciiString,
        sender: GatewaySender,
        gateway_config: Config,
        access_log_sender: ChannelLogSender,
    ) -> crate::Result<Self> {
        let gdn_client = reqwest::ClientBuilder::new()
            .timeout(GDN_TIMEOUT)
            .connect_timeout(CONNECT_TIMEOUT)
            .http2_keep_alive_interval(Some(KEEPALIVE_INTERVAL))
            .http2_keep_alive_timeout(KEEPALIVE_TIMEOUT)
            .http2_keep_alive_while_idle(KEEPALIVE_WHILE_IDLE)
            .user_agent(USER_AGENT)
            .build()
            .map_err(|e| crate::Error::InternalError(e.to_string()))?;

        let gdn_host = match std::env::var("GRAFBASE_GDN_URL") {
            Ok(host) => Cow::Owned(host),
            Err(_) => Cow::Borrowed(GDN_HOST),
        };

        let gdn_url = match graph_ref {
            GraphRef::LatestProductionVersion { graph_slug } => format!("{gdn_host}/graphs/{graph_slug}/current"),
            GraphRef::LatestVersion {
                graph_slug,
                branch_name,
            } => format!("{gdn_host}/graphs/{graph_slug}/{branch_name}/current"),
            GraphRef::Id {
                graph_slug,
                branch_name,
                version,
            } => format!("{gdn_host}/graphs/{graph_slug}/{branch_name}/{version}"),
        };

        let gdn_url = gdn_url
            .parse::<Url>()
            .map_err(|e| crate::Error::InternalError(e.to_string()))?;

        Ok(Self {
            gdn_url,
            gdn_client,
            access_token,
            sender,
            current_id: None,
            gateway_config,
            latencies: meter_from_global_provider()
                .u64_histogram("gdn.request.duration")
                .init(),
            access_log_sender,
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

        // if we have a slow connection, this prevents bursts of connections to the GDN
        // for all the missed ticks.
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {
            interval.tick().await;

            let mut request = self
                .gdn_client
                .get(self.gdn_url.as_str())
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
                        GdnFetchLatencyAttributes {
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
                    GdnFetchLatencyAttributes {
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
                    GdnFetchLatencyAttributes {
                        kind: ResponseKind::GdnError,
                        status_code: e.status(),
                    },
                    duration,
                );

                match e.status() {
                    Some(StatusCode::NOT_FOUND) => {
                        tracing::warn!("Federated schema not found. Is your graph configured as self-hosted? Did you publish at least one subgraph?");
                    }
                    _ => {
                        tracing::error!("Failed to update graph: {e}");
                    }
                }
                continue;
            }

            let response: GdnResponse = match response.json().await {
                Ok(response) => response,
                Err(e) => {
                    self.record_duration(
                        GdnFetchLatencyAttributes {
                            kind: ResponseKind::GdnError,
                            status_code: e.status(),
                        },
                        duration,
                    );

                    tracing::error!("Failed to update graph: {e}");
                    continue;
                }
            };

            tracing::info!("Fetched new Graph");

            let version_id = response.version_id;
            let gateway = match super::gateway::generate(
                GraphDefinition::Gdn(response),
                &self.gateway_config,
                None,
                self.access_log_sender.clone(),
            )
            .await
            {
                Ok(gateway) => gateway,
                Err(e) => {
                    self.record_duration(
                        GdnFetchLatencyAttributes {
                            kind: ResponseKind::GdnError,
                            status_code: None,
                        },
                        duration,
                    );

                    tracing::error!("Failed to process received graph: {e}");
                    continue;
                }
            };

            self.record_duration(
                GdnFetchLatencyAttributes {
                    kind: ResponseKind::New,
                    status_code: None,
                },
                duration,
            );

            self.current_id = Some(version_id);

            self.sender
                .send(Some(Arc::new(gateway)))
                .expect("internal error: channel closed");
        }
    }

    fn record_duration(
        &self,
        GdnFetchLatencyAttributes { kind, status_code }: GdnFetchLatencyAttributes,
        duration: Duration,
    ) {
        let mut attributes = vec![
            KeyValue::new("server.address", self.gdn_url.to_string()),
            KeyValue::new("gdn.response.kind", kind.as_str()),
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
