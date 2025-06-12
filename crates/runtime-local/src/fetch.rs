mod signing;

use std::future::Future;
use std::time::Duration;

use anyhow::bail;
use bytes::Bytes;
use event_queue::{SubgraphResponse, SubgraphResponseBuilder};
use futures_util::Stream;
use futures_util::{StreamExt, TryStreamExt};
use fxhash::FxHashMap;
use gateway_config::Config;
use reqwest::{Certificate, Identity, RequestBuilder};
use reqwest_eventsource::RequestBuilderExt;
use runtime::fetch::{FetchError, FetchRequest, FetchResult, Fetcher};
use signing::SigningParameters;

const POOL_IDLE_TIMEOUT: Duration = Duration::from_secs(5);
const ENABLE_HICKORY_DNS: bool = true;

#[derive(Clone)]
pub struct NativeFetcher {
    client: reqwest::Client,
    dedicated_clients: FxHashMap<String, reqwest::Client>,
    default_signing_parameters: Option<SigningParameters>,
    subgraph_signing_parameters: FxHashMap<String, Option<SigningParameters>>,
}

impl NativeFetcher {
    pub fn new(config: &Config) -> anyhow::Result<Self> {
        let default_signing_params = SigningParameters::from_config(&config.gateway.message_signatures, None)?;
        let subgraph_signing_parameters = generate_subgraph_signing_parameters(config)?;
        let dedicated_clients = generate_dedicated_http_clients(config)?;

        Ok(NativeFetcher {
            client: reqwest::Client::builder()
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
                .build()?,
            default_signing_parameters: default_signing_params,
            subgraph_signing_parameters,
            dedicated_clients,
        })
    }

    pub fn client(&self, subgraph_name: &str) -> &reqwest::Client {
        self.dedicated_clients.get(subgraph_name).unwrap_or(&self.client)
    }
}

impl Fetcher for NativeFetcher {
    async fn fetch(
        &self,
        fetch_req: FetchRequest<'_, Bytes>,
    ) -> (FetchResult<http::Response<Bytes>>, Option<SubgraphResponseBuilder>) {
        let mut info = SubgraphResponse::builder();

        let subgraph_name = fetch_req.subgraph_name;
        let request = into_reqwest(fetch_req);

        let request = match self.sign_request(subgraph_name, request).await {
            Ok(request) => request,
            Err(error) => return (Err(error), None),
        };

        let result = self.client(subgraph_name).execute(request).await.map_err(Into::into);
        info.track_connection();

        let mut resp = match result {
            Ok(response) => response,
            Err(e) => {
                return (Err(e), Some(info));
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
            Err(e) => return (Err(e.into()), Some(info)),
        };

        // reqwest transforms the body into a stream with Into
        let mut response = http::Response::new(bytes);
        *response.status_mut() = status;
        *response.version_mut() = version;
        *response.extensions_mut() = extensions;
        *response.headers_mut() = headers;

        (Ok(response), Some(info))
    }

    async fn graphql_over_sse_stream(
        &self,
        request: FetchRequest<'_, Bytes>,
    ) -> FetchResult<impl Stream<Item = FetchResult<Bytes>> + Send + 'static> {
        let mut request = into_reqwest(request);
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
        request: FetchRequest<'_, T>,
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
                .subscribe(WebsocketRequest(body?))
                .await
                .map_err(|err| err.to_string())?
                .map(|item| item.map_err(|err| FetchError::from(err.to_string()))))
        }
    }
}

fn into_reqwest(request: FetchRequest<'_, Bytes>) -> reqwest::Request {
    let mut req = reqwest::Request::new(request.method, request.url.into_owned());
    *req.headers_mut() = request.headers;
    *req.body_mut() = Some(request.body.into());
    *req.timeout_mut() = Some(request.timeout);
    req
}

/// Creates a HashMap of dedicated HTTP clients for subgraphs that require mTLS.
fn generate_dedicated_http_clients(config: &Config) -> anyhow::Result<FxHashMap<String, reqwest::Client>> {
    let mut clients = FxHashMap::default();

    for (name, config) in &config.subgraphs {
        let Some(mtls_config) = config.mtls.as_ref() else {
            continue;
        };

        if mtls_config.root.is_none() && mtls_config.identity.is_none() {
            continue;
        }

        let mut builder = reqwest::Client::builder()
            .pool_idle_timeout(Some(POOL_IDLE_TIMEOUT))
            .hickory_dns(ENABLE_HICKORY_DNS)
            .danger_accept_invalid_certs(mtls_config.accept_invalid_certs);

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
            clients.insert(name.clone(), builder.build()?);
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
        clients.insert(name.clone(), builder.build()?);
    }

    Ok(clients)
}

fn generate_subgraph_signing_parameters(
    config: &Config,
) -> Result<FxHashMap<String, Option<SigningParameters>>, anyhow::Error> {
    let subgraph_signing_parameters = config
        .subgraphs
        .iter()
        .filter_map(|(name, value)| Some((name, value.message_signatures.as_ref()?)))
        .map(|(name, message_signatures)| {
            Ok((
                name.clone(),
                SigningParameters::from_config(message_signatures, Some(&config.gateway.message_signatures))?,
            ))
        })
        .collect::<anyhow::Result<_>>()?;

    Ok(subgraph_signing_parameters)
}

#[derive(serde::Serialize)]
struct WebsocketRequest<T>(T);

impl<T: serde::Serialize> graphql_ws_client::graphql::GraphqlOperation for WebsocketRequest<T> {
    type Response = serde_json::Value;

    type Error = FetchError;

    fn decode(&self, data: serde_json::Value) -> Result<Self::Response, Self::Error> {
        Ok(data)
    }
}
