use std::time::Duration;

use ascii::AsciiString;
use http::{HeaderName, HeaderValue};
use tower_http::cors::{AllowHeaders, AllowMethods, AllowOrigin, ExposeHeaders, MaxAge};
use url::Url;

use crate::errors::CliError;

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CorsConfig {
    /// If false (or not defined), credentials are not allowed in requests
    #[serde(default)]
    pub allow_credentials: bool,
    /// Origins from which we allow requests
    pub allow_origins: Option<AnyOrUrlArray>,
    /// Maximum time between OPTIONS and the next request
    pub max_age: Option<u64>,
    /// HTTP methods allowed to the endpoint.
    pub allow_methods: Option<AnyOrHttpMethodArray>,
    /// Headers allowed in incoming requests
    pub allow_headers: Option<AnyOrAsciiStringArray>,
    /// Headers exposed from the OPTIONS request
    pub expose_headers: Option<AnyOrAsciiStringArray>,
    /// If set, allows browsers from private network to connect
    #[serde(default)]
    pub allow_private_network: bool,
}

#[derive(Debug, PartialEq, Clone, Copy, serde::Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Head,
    Options,
    Connect,
    Patch,
    Trace,
}

impl From<HttpMethod> for http::Method {
    fn from(value: HttpMethod) -> Self {
        match value {
            HttpMethod::Get => http::Method::GET,
            HttpMethod::Post => http::Method::POST,
            HttpMethod::Put => http::Method::PUT,
            HttpMethod::Delete => http::Method::DELETE,
            HttpMethod::Head => http::Method::HEAD,
            HttpMethod::Options => http::Method::OPTIONS,
            HttpMethod::Connect => http::Method::CONNECT,
            HttpMethod::Patch => http::Method::PATCH,
            HttpMethod::Trace => http::Method::TRACE,
        }
    }
}

#[derive(Debug, PartialEq, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AnyOrUrlArray {
    Any,
    #[serde(untagged)]
    Explicit(Vec<Url>),
}

impl From<AnyOrUrlArray> for AllowOrigin {
    fn from(value: AnyOrUrlArray) -> Self {
        match value {
            AnyOrUrlArray::Any => AllowOrigin::any(),
            AnyOrUrlArray::Explicit(ref origins) => {
                let origins = origins
                    .iter()
                    .map(|origin| HeaderValue::from_str(origin.as_str()).expect("must be ascii"));

                AllowOrigin::list(origins)
            }
        }
    }
}

#[derive(Debug, PartialEq, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AnyOrHttpMethodArray {
    Any,
    #[serde(untagged)]
    Explicit(Vec<HttpMethod>),
}

impl From<AnyOrHttpMethodArray> for AllowMethods {
    fn from(value: AnyOrHttpMethodArray) -> Self {
        match value {
            AnyOrHttpMethodArray::Any => AllowMethods::any(),
            AnyOrHttpMethodArray::Explicit(methods) => {
                let methods = methods.iter().map(|method| http::Method::from(*method));
                AllowMethods::list(methods)
            }
        }
    }
}

#[derive(Debug, PartialEq, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AnyOrAsciiStringArray {
    Any,
    #[serde(untagged)]
    Explicit(Vec<AsciiString>),
}

impl From<AnyOrAsciiStringArray> for AllowHeaders {
    fn from(value: AnyOrAsciiStringArray) -> Self {
        match value {
            AnyOrAsciiStringArray::Any => AllowHeaders::any(),
            AnyOrAsciiStringArray::Explicit(headers) => {
                let headers = headers
                    .iter()
                    .map(|header| HeaderName::from_bytes(header.as_bytes()).expect("must be ascii"));

                AllowHeaders::list(headers)
            }
        }
    }
}

impl From<AnyOrAsciiStringArray> for ExposeHeaders {
    fn from(value: AnyOrAsciiStringArray) -> Self {
        match value {
            AnyOrAsciiStringArray::Any => ExposeHeaders::any(),
            AnyOrAsciiStringArray::Explicit(headers) => {
                let headers = headers
                    .iter()
                    .map(|header| HeaderName::from_bytes(header.as_bytes()).expect("must be ascii"));

                ExposeHeaders::list(headers)
            }
        }
    }
}
