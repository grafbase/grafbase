use wasmtime::component::Resource;

use crate::{
    WasiState,
    extension::wit::{HostHttpClient, HttpClient, HttpMethod, HttpVersion},
    http_client::{HttpError, HttpRequest, HttpResponse},
};

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
