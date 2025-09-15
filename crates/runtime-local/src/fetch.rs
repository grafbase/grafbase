mod signing;
mod traffic_shaping;

use std::future::Future;
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;

use anyhow::bail;
use bytes::Bytes;
use engine::Schema;
use engine_schema::GraphqlSubgraphId;
use event_queue::{SubgraphResponse, SubgraphResponseBuilder};
use futures_util::Stream;
use futures_util::{StreamExt, TryStreamExt};
use fxhash::FxHashMap;
use gateway_config::Config;
use rapidhash::fast::RapidHashMap;
use reqwest::{Certificate, Identity, RequestBuilder};
use reqwest_eventsource::RequestBuilderExt;
use runtime::fetch::{FetchError, FetchRequest, FetchResult, Fetcher, WebsocketRequest};

use crate::fetch::traffic_shaping::TrafficShaping;

const POOL_IDLE_TIMEOUT: Duration = Duration::from_secs(5);
const ENABLE_HICKORY_DNS: bool = true;

pub struct NativeFetcherInner {
    client: reqwest::Client,
    signer: signing::RequestSigner,
    dedicated_clients: FxHashMap<GraphqlSubgraphId, reqwest::Client>,
    traffic_shaping: traffic_shaping::TrafficShaping,
}

#[derive(Clone)]
pub struct NativeFetcher(Arc<NativeFetcherInner>);

impl Deref for NativeFetcher {
    type Target = NativeFetcherInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl NativeFetcher {
    pub fn new(config: &Config, schema: &Schema) -> anyhow::Result<Self> {
        let name_to_id = schema
            .graphql_subgraphs()
            .map(|s| (s.name(), s.id))
            .collect::<RapidHashMap<_, _>>();
        let signer = signing::RequestSigner::new(config, &name_to_id)?;
        let dedicated_clients = generate_dedicated_http_clients(config, &name_to_id)?;

        Ok(NativeFetcher(Arc::new(NativeFetcherInner {
            client: client_builder().build()?,
            signer,
            dedicated_clients,
            traffic_shaping: TrafficShaping::new(&config.traffic_shaping),
        })))
    }
}

pub fn client_builder() -> reqwest::ClientBuilder {
    reqwest::Client::builder()
        // Hyper connection pool only exposes two parameters max idle connections per host
        // and idle connection timeout. There is not TTL on the connections themselves to
        // force a refresh, necessary if the DNS changes its records. Somehow, even within
        // a benchmark ramping *up* traffic, we do pick up DNS changes by setting a pool
        // idle timeout of 5 seconds even though in theory no connection should be idle?
        // A bit confusing, and I suspect I don't fully understand how Hyper is managing
        // connections underneath. But seems like best choice we have right now, Apollo'
        // router uses this same default value.
        .pool_idle_timeout(Some(POOL_IDLE_TIMEOUT))
        .hickory_dns(ENABLE_HICKORY_DNS)
        .tcp_nodelay(true)
        .tcp_keepalive(Some(std::time::Duration::from_secs(60)))
}

#[derive(Clone)]
struct FetchResponse {
    result: FetchResult<http::Response<Bytes>>,
    info: Option<SubgraphResponseBuilder>,
}

impl NativeFetcherInner {
    async fn execute(&self, fetch_req: FetchRequest<'_>) -> FetchResponse {
        let mut info = SubgraphResponse::builder();

        let subgraph_id = fetch_req.subgraph_id;
        let request = into_reqwest(fetch_req);

        let request = match self.signer.sign(subgraph_id, request).await {
            Ok(request) => request,
            Err(error) => {
                return FetchResponse {
                    result: Err(error),
                    info: None,
                };
            }
        };

        let result = self.client(subgraph_id).execute(request).await.map_err(Into::into);
        info.track_connection();

        let mut resp = match result {
            Ok(response) => response,
            Err(e) => {
                return FetchResponse {
                    result: Err(e),
                    info: Some(info),
                };
            }
        };

        let status = resp.status();
        let headers = std::mem::take(resp.headers_mut());
        let extensions = std::mem::take(resp.extensions_mut());
        let version = resp.version();
        let result = resp.bytes().await;

        info.track_response();

        let bytes = match result {
            Ok(bytes) => bytes,
            Err(e) => {
                return FetchResponse {
                    result: Err(e.into()),
                    info: Some(info),
                };
            }
        };

        // reqwest transforms the body into a stream with Into
        let mut response = http::Response::new(bytes);
        *response.status_mut() = status;
        *response.version_mut() = version;
        *response.extensions_mut() = extensions;
        *response.headers_mut() = headers;

        FetchResponse {
            result: Ok(response),
            info: Some(info),
        }
    }

    fn client(&self, subgraph_id: GraphqlSubgraphId) -> &reqwest::Client {
        self.dedicated_clients.get(&subgraph_id).unwrap_or(&self.client)
    }
}

impl Fetcher for NativeFetcher {
    async fn fetch(
        &self,
        request: FetchRequest<'_>,
    ) -> (FetchResult<http::Response<Bytes>>, Option<SubgraphResponseBuilder>) {
        let FetchResponse { result, info } = self
            .traffic_shaping
            .deduplicate(request, |request| async move { self.execute(request).await })
            .await;
        (result, info)
    }

    async fn graphql_over_sse_stream(
        &self,
        request: WebsocketRequest<'_, Bytes>,
    ) -> FetchResult<impl Stream<Item = FetchResult<Bytes>> + Send + 'static> {
        let mut request = ws_to_reqwest(request);
        // We're doing a streaming request, for subscriptions, so we don't want to timeout
        *request.timeout_mut() = None;

        let events = RequestBuilder::from_parts(self.client.clone(), request)
            .eventsource()
            .unwrap()
            .map_err(|err| match err {
                reqwest_eventsource::Error::InvalidStatusCode(status_code, _) => {
                    FetchError::InvalidStatusCode(status_code)
                }
                err => FetchError::Message(err.to_string()),
            })
            .try_take_while(|event| {
                let is_complete = if let reqwest_eventsource::Event::Message(message) = event {
                    message.event == "complete"
                } else {
                    false
                };
                async move { Ok(!is_complete) }
            })
            .try_filter_map(|event| async move {
                let reqwest_eventsource::Event::Message(message) = event else {
                    return Ok(None);
                };
                if message.event == "next" {
                    Ok(Some(message.data.into()))
                } else {
                    Err(FetchError::Message(format!("Unexpected event: {}", message.event)))
                }
            });
        Ok(events)
    }

    fn graphql_over_websocket_stream<T>(
        &self,
        request: WebsocketRequest<'_, T>,
    ) -> impl Future<Output = FetchResult<impl Stream<Item = FetchResult<serde_json::Value>> + Send + 'static>> + Send
    where
        T: serde::Serialize + Send,
    {
        use tungstenite::{client::IntoClientRequest, http::HeaderValue};

        // graphql_ws_client requires a 'static body which we can't provide.
        let body = serde_json::value::to_raw_value(&request.body).map_err(|err| err.to_string());
        let mut ws_request = request.url.as_ref().into_client_request().unwrap();
        ws_request.headers_mut().extend(request.headers);
        ws_request.headers_mut().insert(
            http::header::SEC_WEBSOCKET_PROTOCOL,
            HeaderValue::from_str("graphql-transport-ws").unwrap(),
        );

        async move {
            let (connection, _) = {
                async_tungstenite::tokio::connect_async(ws_request)
                    .await
                    .map_err(|err| err.to_string())?
            };

            Ok(graphql_ws_client::Client::build(connection)
                .payload(request.websocket_init_payload)
                .map_err(|err| err.to_string())?
                .subscribe(GraphqlWsRequest(body?))
                .await
                .map_err(|err| err.to_string())?
                .map(|item| item.map_err(|err| FetchError::from(err.to_string()))))
        }
    }
}

fn into_reqwest(request: FetchRequest<'_>) -> reqwest::Request {
    let mut req = reqwest::Request::new(request.method, request.url.into_owned());
    *req.headers_mut() = request.headers;
    *req.body_mut() = Some(request.body.into());
    *req.timeout_mut() = Some(request.timeout);
    req
}

fn ws_to_reqwest(request: WebsocketRequest<'_, Bytes>) -> reqwest::Request {
    let mut req = reqwest::Request::new(request.method, request.url.into_owned());
    *req.headers_mut() = request.headers;
    *req.body_mut() = Some(request.body.into());
    *req.timeout_mut() = Some(request.timeout);
    req
}

/// Creates a HashMap of dedicated HTTP clients for subgraphs that require mTLS.
fn generate_dedicated_http_clients(
    config: &Config,
    subgraph_name_to_id: &RapidHashMap<&str, GraphqlSubgraphId>,
) -> anyhow::Result<FxHashMap<GraphqlSubgraphId, reqwest::Client>> {
    let mut clients = FxHashMap::default();

    for (name, config) in &config.subgraphs {
        let Some(mtls_config) = config.mtls.as_ref() else {
            continue;
        };

        if mtls_config.root.is_none() && mtls_config.identity.is_none() {
            continue;
        }

        let id = match subgraph_name_to_id.get(name.as_str()) {
            Some(id) => *id,
            None => {
                continue;
            }
        };

        let mut builder = client_builder().danger_accept_invalid_certs(mtls_config.accept_invalid_certs);

        if let Some(ref root) = mtls_config.root {
            let ca_cert_bytes = match std::fs::read(&root.certificate) {
                Ok(bytes) => bytes,
                Err(e) => {
                    bail!(
                        "failed to open root certificate `{}` for subgraph `{name}`: {e}",
                        root.certificate.display()
                    );
                }
            };

            if root.is_bundle {
                let certificates = match Certificate::from_pem_bundle(&ca_cert_bytes) {
                    Ok(certificates) => certificates,
                    Err(e) => {
                        bail!(
                            "failed to parse root certificate `{}` for subgraph `{name}`: {e}",
                            root.certificate.display()
                        );
                    }
                };

                for certificate in certificates {
                    builder = builder.add_root_certificate(certificate);
                }
            } else {
                let certificate = match Certificate::from_pem(&ca_cert_bytes) {
                    Ok(certificate) => certificate,
                    Err(e) => {
                        bail!(
                            "failed to parse root certificate `{}` for subgraph `{name}`: {e}",
                            root.certificate.display()
                        );
                    }
                };

                builder = builder.add_root_certificate(certificate);
            };
        }

        let Some(ref identity_path) = mtls_config.identity else {
            clients.insert(id, builder.build()?);
            continue;
        };

        let identity = match std::fs::read(identity_path) {
            Ok(identity) => identity,
            Err(e) => {
                bail!(
                    "failed to read identity file `{}` for subgraph `{name}`: {e}",
                    identity_path.display()
                );
            }
        };

        let identity = match Identity::from_pem(&identity) {
            Ok(identity) => identity,
            Err(e) => {
                bail!(
                    "failed to parse identity file `{}` for subgraph `{name}`: {e}",
                    identity_path.display()
                )
            }
        };

        builder = builder.identity(identity);
        clients.insert(id, builder.build()?);
    }

    Ok(clients)
}

#[derive(serde::Serialize)]
struct GraphqlWsRequest<T>(T);

impl<T: serde::Serialize> graphql_ws_client::graphql::GraphqlOperation for GraphqlWsRequest<T> {
    type Response = serde_json::Value;

    type Error = FetchError;

    fn decode(&self, data: serde_json::Value) -> Result<Self::Response, Self::Error> {
        Ok(data)
    }
}
