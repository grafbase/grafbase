use super::super::wit::grafbase::sdk::types;
use futures::TryFutureExt;
use wasmtime::component::Resource;

use crate::WasiState;

impl types::HostHttpClient for WasiState {
    async fn execute(
        &mut self,
        request: types::HttpRequest,
    ) -> wasmtime::Result<Result<types::HttpResponse, types::HttpError>> {
        if !self.network_enabled() {
            return Ok(Err(types::HttpError::Connect("Network is disabled".into())));
        }

        let request_durations = self.request_durations().clone();
        let http_client = self.http_client().clone();

        let response = crate::http_client::send_request(http_client, request_durations, request.into()).await;

        Ok(response.map(Into::into).map_err(Into::into))
    }

    async fn execute_many(
        &mut self,
        requests: Vec<types::HttpRequest>,
    ) -> wasmtime::Result<Vec<Result<types::HttpResponse, types::HttpError>>> {
        if !self.network_enabled() {
            return Ok(vec![
                Err(types::HttpError::Connect("Network is disabled".into()));
                requests.len()
            ]);
        }

        let request_durations = self.request_durations().clone();
        let http_client = self.http_client().clone();

        let futures = requests
            .into_iter()
            .map(|request| {
                let future =
                    crate::http_client::send_request(http_client.clone(), request_durations.clone(), request.into());

                future.map_ok(Into::into).map_err(Into::into)
            })
            .collect::<Vec<_>>();

        Ok(futures::future::join_all(futures).await)
    }

    async fn drop(&mut self, _: Resource<types::HttpClient>) -> wasmtime::Result<()> {
        // Singleton that is never allocated
        Ok(())
    }
}

impl From<types::HttpMethod> for reqwest::Method {
    fn from(value: types::HttpMethod) -> Self {
        match value {
            types::HttpMethod::Get => reqwest::Method::GET,
            types::HttpMethod::Post => reqwest::Method::POST,
            types::HttpMethod::Put => reqwest::Method::PUT,
            types::HttpMethod::Delete => reqwest::Method::DELETE,
            types::HttpMethod::Patch => reqwest::Method::PATCH,
            types::HttpMethod::Head => reqwest::Method::HEAD,
            types::HttpMethod::Options => reqwest::Method::OPTIONS,
            types::HttpMethod::Connect => reqwest::Method::CONNECT,
            types::HttpMethod::Trace => reqwest::Method::TRACE,
        }
    }
}

impl From<http::Method> for types::HttpMethod {
    fn from(value: http::Method) -> Self {
        match value {
            http::Method::GET => types::HttpMethod::Get,
            http::Method::POST => types::HttpMethod::Post,
            http::Method::PUT => types::HttpMethod::Put,
            http::Method::DELETE => types::HttpMethod::Delete,
            http::Method::PATCH => types::HttpMethod::Patch,
            http::Method::HEAD => types::HttpMethod::Head,
            http::Method::OPTIONS => types::HttpMethod::Options,
            http::Method::CONNECT => types::HttpMethod::Connect,
            http::Method::TRACE => types::HttpMethod::Trace,
            method => todo!("unsupported http method: {method:?}"),
        }
    }
}

impl From<reqwest::Version> for types::HttpVersion {
    fn from(value: reqwest::Version) -> Self {
        match value {
            reqwest::Version::HTTP_09 => types::HttpVersion::Http09,
            reqwest::Version::HTTP_10 => types::HttpVersion::Http10,
            reqwest::Version::HTTP_11 => types::HttpVersion::Http11,
            reqwest::Version::HTTP_2 => types::HttpVersion::Http20,
            reqwest::Version::HTTP_3 => types::HttpVersion::Http30,
            version => todo!("unsupported http version: {version:?}"),
        }
    }
}

impl AsRef<str> for types::HttpMethod {
    fn as_ref(&self) -> &str {
        match self {
            types::HttpMethod::Get => "GET",
            types::HttpMethod::Post => "POST",
            types::HttpMethod::Put => "PUT",
            types::HttpMethod::Delete => "DELETE",
            types::HttpMethod::Patch => "PATCH",
            types::HttpMethod::Head => "HEAD",
            types::HttpMethod::Options => "OPTIONS",
            types::HttpMethod::Connect => "CONNECT",
            types::HttpMethod::Trace => "TRACE",
        }
    }
}
