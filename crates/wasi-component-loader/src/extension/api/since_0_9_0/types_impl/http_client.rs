use anyhow::bail;
use wasmtime::component::Resource;

use super::super::wit::http_client;
use crate::WasiState;

impl http_client::HostHttpClient for WasiState {
    async fn execute(
        &mut self,
        request: http_client::HttpRequest,
    ) -> wasmtime::Result<Result<http_client::HttpResponse, http_client::HttpError>> {
        if !self.network_enabled() {
            bail!("Network operations are disabled");
        }

        let request_durations = self.request_durations().clone();
        let http_client = self.http_client().clone();

        let response = crate::http_client::send_request(http_client, request_durations, request).await;

        Ok(response)
    }

    async fn execute_many(
        &mut self,
        requests: Vec<http_client::HttpRequest>,
    ) -> wasmtime::Result<Vec<Result<http_client::HttpResponse, http_client::HttpError>>> {
        if !self.network_enabled() {
            bail!("Network operations are disabled");
        }

        let request_durations = self.request_durations().clone();
        let http_client = self.http_client().clone();

        let futures = requests
            .into_iter()
            .map(|request| crate::http_client::send_request(http_client.clone(), request_durations.clone(), request))
            .collect::<Vec<_>>();

        Ok(futures::future::join_all(futures).await)
    }

    async fn drop(&mut self, _: Resource<http_client::HttpClient>) -> wasmtime::Result<()> {
        // Singleton that is never allocated
        Ok(())
    }
}

impl From<http_client::HttpMethod> for reqwest::Method {
    fn from(value: http_client::HttpMethod) -> Self {
        match value {
            http_client::HttpMethod::Get => reqwest::Method::GET,
            http_client::HttpMethod::Post => reqwest::Method::POST,
            http_client::HttpMethod::Put => reqwest::Method::PUT,
            http_client::HttpMethod::Delete => reqwest::Method::DELETE,
            http_client::HttpMethod::Patch => reqwest::Method::PATCH,
            http_client::HttpMethod::Head => reqwest::Method::HEAD,
            http_client::HttpMethod::Options => reqwest::Method::OPTIONS,
            http_client::HttpMethod::Connect => reqwest::Method::CONNECT,
            http_client::HttpMethod::Trace => reqwest::Method::TRACE,
        }
    }
}

impl From<http::Method> for http_client::HttpMethod {
    fn from(value: http::Method) -> Self {
        match value {
            http::Method::GET => http_client::HttpMethod::Get,
            http::Method::POST => http_client::HttpMethod::Post,
            http::Method::PUT => http_client::HttpMethod::Put,
            http::Method::DELETE => http_client::HttpMethod::Delete,
            http::Method::PATCH => http_client::HttpMethod::Patch,
            http::Method::HEAD => http_client::HttpMethod::Head,
            http::Method::OPTIONS => http_client::HttpMethod::Options,
            http::Method::CONNECT => http_client::HttpMethod::Connect,
            http::Method::TRACE => http_client::HttpMethod::Trace,
            method => todo!("unsupported http method: {method:?}"),
        }
    }
}

impl From<reqwest::Version> for http_client::HttpVersion {
    fn from(value: reqwest::Version) -> Self {
        match value {
            reqwest::Version::HTTP_09 => http_client::HttpVersion::Http09,
            reqwest::Version::HTTP_10 => http_client::HttpVersion::Http10,
            reqwest::Version::HTTP_11 => http_client::HttpVersion::Http11,
            reqwest::Version::HTTP_2 => http_client::HttpVersion::Http20,
            reqwest::Version::HTTP_3 => http_client::HttpVersion::Http30,
            version => todo!("unsupported http version: {version:?}"),
        }
    }
}

impl AsRef<str> for http_client::HttpMethod {
    fn as_ref(&self) -> &str {
        match self {
            http_client::HttpMethod::Get => "GET",
            http_client::HttpMethod::Post => "POST",
            http_client::HttpMethod::Put => "PUT",
            http_client::HttpMethod::Delete => "DELETE",
            http_client::HttpMethod::Patch => "PATCH",
            http_client::HttpMethod::Head => "HEAD",
            http_client::HttpMethod::Options => "OPTIONS",
            http_client::HttpMethod::Connect => "CONNECT",
            http_client::HttpMethod::Trace => "TRACE",
        }
    }
}
