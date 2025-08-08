use crate::{types::Headers, wit};

/// Represents the parts of an HTTP request, including the URL, method, and headers.
#[non_exhaustive]
pub struct HttpRequestParts {
    /// The URL of the HTTP request.
    pub url: String,
    /// The HTTP method of the request, such as GET, POST, etc.
    pub method: http::Method,
    /// The headers of the HTTP request.
    pub headers: Headers,
}

impl From<wit::HttpRequestParts> for HttpRequestParts {
    fn from(parts: wit::HttpRequestParts) -> Self {
        Self {
            url: parts.url,
            method: parts.method.into(),
            headers: parts.headers.into(),
        }
    }
}

impl From<HttpRequestParts> for wit::HttpRequestParts {
    fn from(parts: HttpRequestParts) -> Self {
        Self {
            url: parts.url,
            method: parts.method.into(),
            headers: parts.headers.into(),
        }
    }
}

/// Output type for the [on_request()](crate::HooksExtension::on_request()) hook.
#[derive(Default)]
pub struct OnRequestOutput {
    pub(crate) state: Vec<u8>,
    pub(crate) contract_key: Option<String>,
}

impl OnRequestOutput {
    /// Creates a new [OnRequestOutput] instance with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the contract key for the request.
    pub fn contract_key(mut self, contract_key: impl Into<String>) -> Self {
        self.contract_key = Some(contract_key.into());
        self
    }
}
