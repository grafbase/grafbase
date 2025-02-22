use async_nats::ServerAddr;
use wasmtime::component::Resource;

use crate::{AccessLogMessage, state::WasiState};

use super::wit::*;

impl Host for WasiState {}

impl HostSharedContext for WasiState {
    async fn get(&mut self, self_: Resource<SharedContext>, name: String) -> wasmtime::Result<Option<String>> {
        let ctx = WasiState::get(self, &self_)?;
        Ok(ctx.kv.get(&name).cloned())
    }

    async fn trace_id(&mut self, self_: Resource<SharedContext>) -> wasmtime::Result<String> {
        let ctx = WasiState::get(self, &self_)?;
        Ok(ctx.trace_id.to_string())
    }

    async fn drop(&mut self, rep: Resource<SharedContext>) -> wasmtime::Result<()> {
        self.table.delete(rep)?;
        Ok(())
    }
}

impl HostAccessLog for WasiState {
    async fn send(&mut self, data: Vec<u8>) -> wasmtime::Result<Result<(), LogError>> {
        let data = AccessLogMessage::Data(data);

        Ok(self.access_log().send(data).inspect_err(|err| match err {
            LogError::ChannelFull(_) => {
                tracing::error!("access log channel is over capacity");
            }
            LogError::ChannelClosed => {
                tracing::error!("access log channel closed");
            }
        }))
    }

    async fn drop(&mut self, _: Resource<AccessLog>) -> wasmtime::Result<()> {
        // Singleton that is never allocated
        Ok(())
    }
}

impl HostNatsClient for WasiState {
    async fn connect(
        &mut self,
        servers: Vec<String>,
        auth: Option<NatsAuth>,
    ) -> wasmtime::Result<Result<Resource<NatsClient>, String>> {
        let Ok(addrs) = servers
            .iter()
            .map(|url| url.parse())
            .collect::<Result<Vec<ServerAddr>, _>>()
        else {
            return Ok(Err("Failed to parse server URLs".to_string()));
        };

        let opts = async_nats::ConnectOptions::new();

        let opts = match auth {
            Some(NatsAuth::UsernamePassword((username, password))) => opts.user_and_password(username, password),
            Some(NatsAuth::Token(token)) => opts.token(token),
            Some(NatsAuth::Credentials(ref credentials)) => match opts.credentials(credentials) {
                Ok(opts) => opts,
                Err(err) => return Ok(Err(err.to_string())),
            },
            None => opts,
        };

        Ok(match async_nats::connect_with_options(addrs, opts).await {
            Ok(client) => {
                let client = self.push_resource(client)?;

                Ok(client)
            }
            Err(err) => Err(err.to_string()),
        })
    }

    async fn publish(
        &mut self,
        self_: Resource<NatsClient>,
        subject: String,
        message: Vec<u8>,
    ) -> wasmtime::Result<Result<(), String>> {
        let client = self.get_mut(&self_)?;
        Ok(client
            .publish(subject, message.into())
            .await
            .map_err(|err| err.to_string()))
    }

    async fn drop(&mut self, rep: Resource<NatsClient>) -> wasmtime::Result<()> {
        self.table.delete(rep)?;
        Ok(())
    }
}

impl HostHttpClient for WasiState {
    async fn execute(&mut self, request: HttpRequest) -> wasmtime::Result<Result<HttpResponse, HttpError>> {
        let request_durations = self.request_durations().clone();
        let http_client = self.http_client().clone();

        Ok(crate::http_client::send_request(http_client, request_durations, request).await)
    }

    async fn execute_many(
        &mut self,
        requests: Vec<HttpRequest>,
    ) -> wasmtime::Result<Vec<Result<HttpResponse, HttpError>>> {
        let request_durations = self.request_durations().clone();
        let http_client = self.http_client().clone();

        let futures = requests
            .into_iter()
            .map(|request| crate::http_client::send_request(http_client.clone(), request_durations.clone(), request))
            .collect::<Vec<_>>();

        Ok(futures::future::join_all(futures).await)
    }

    async fn drop(&mut self, _: Resource<HttpClient>) -> wasmtime::Result<()> {
        // Singleton that is never allocated
        Ok(())
    }
}

impl HostHeaders for WasiState {
    async fn get(&mut self, self_: Resource<Headers>, name: String) -> wasmtime::Result<Option<String>> {
        let headers = WasiState::get_ref(self, &self_)?;
        Ok(headers
            .get(&name)
            .map(|val| String::from_utf8_lossy(val.as_bytes()).into_owned()))
    }

    async fn drop(&mut self, rep: Resource<Headers>) -> wasmtime::Result<()> {
        if !WasiState::get(self, &rep)?.is_host_borrowed() {
            self.table.delete(rep)?;
        }
        Ok(())
    }
}

impl HostCache for WasiState {
    async fn get(&mut self, key: String) -> wasmtime::Result<Option<Vec<u8>>> {
        Ok(self.cache().get(&key).await)
    }

    async fn set(&mut self, key: String, value: Vec<u8>, ttl_ms: Option<u64>) -> wasmtime::Result<()> {
        self.cache().set(&key, value, ttl_ms).await;
        Ok(())
    }

    async fn drop(&mut self, rep: Resource<Cache>) -> wasmtime::Result<()> {
        self.table.delete(rep)?;
        Ok(())
    }
}

impl From<HttpMethod> for reqwest::Method {
    fn from(value: HttpMethod) -> Self {
        match value {
            HttpMethod::Get => reqwest::Method::GET,
            HttpMethod::Post => reqwest::Method::POST,
            HttpMethod::Put => reqwest::Method::PUT,
            HttpMethod::Delete => reqwest::Method::DELETE,
            HttpMethod::Patch => reqwest::Method::PATCH,
            HttpMethod::Head => reqwest::Method::HEAD,
            HttpMethod::Options => reqwest::Method::OPTIONS,
            HttpMethod::Connect => reqwest::Method::CONNECT,
            HttpMethod::Trace => reqwest::Method::TRACE,
        }
    }
}

impl From<http::Method> for HttpMethod {
    fn from(value: http::Method) -> Self {
        match value {
            http::Method::GET => HttpMethod::Get,
            http::Method::POST => HttpMethod::Post,
            http::Method::PUT => HttpMethod::Put,
            http::Method::DELETE => HttpMethod::Delete,
            http::Method::PATCH => HttpMethod::Patch,
            http::Method::HEAD => HttpMethod::Head,
            http::Method::OPTIONS => HttpMethod::Options,
            http::Method::CONNECT => HttpMethod::Connect,
            http::Method::TRACE => HttpMethod::Trace,
            method => todo!("unsupported http method: {method:?}"),
        }
    }
}

impl From<reqwest::Version> for HttpVersion {
    fn from(value: reqwest::Version) -> Self {
        match value {
            reqwest::Version::HTTP_09 => HttpVersion::Http09,
            reqwest::Version::HTTP_10 => HttpVersion::Http10,
            reqwest::Version::HTTP_11 => HttpVersion::Http11,
            reqwest::Version::HTTP_2 => HttpVersion::Http20,
            reqwest::Version::HTTP_3 => HttpVersion::Http30,
            version => todo!("unsupported http version: {version:?}"),
        }
    }
}

impl AsRef<str> for HttpMethod {
    fn as_ref(&self) -> &str {
        match self {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Delete => "DELETE",
            HttpMethod::Patch => "PATCH",
            HttpMethod::Head => "HEAD",
            HttpMethod::Options => "OPTIONS",
            HttpMethod::Connect => "CONNECT",
            HttpMethod::Trace => "TRACE",
        }
    }
}

impl From<extension_catalog::KindDiscriminants> for ExtensionType {
    fn from(value: extension_catalog::KindDiscriminants) -> Self {
        match value {
            extension_catalog::KindDiscriminants::FieldResolver => ExtensionType::Resolver,
            extension_catalog::KindDiscriminants::Authenticator => ExtensionType::Authentication,
        }
    }
}
