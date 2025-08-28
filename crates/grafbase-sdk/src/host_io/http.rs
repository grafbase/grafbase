//! A module for executing HTTP requests.

use std::{string::FromUtf8Error, time::Duration};

use crate::{types::Headers, wit::HttpMethod};
pub use http::{HeaderName, HeaderValue, Method, StatusCode};
pub use serde_json::Error as JsonDeserializeError;
pub use url::Url;

use crate::{
    types::{AsHeaderName, AsHeaderValue},
    wit::{self, HttpClient},
};
use serde::Serialize;

/// Executes a single HTTP request and returns a result containing either an `HttpResponse` or an `HttpError`.
///
/// This function delegates the execution of the HTTP request to the underlying `HttpClient`, which handles
/// the asynchronous sending of the request in the host runtime. From the perspective of the guest, this operation is blocking.
/// While awaiting the response, other tasks can be executed concurrently by the host thread.
///
/// # Arguments
///
/// * `request` - A reference to an `HttpRequest` that encapsulates the HTTP method, URL, headers,
///   body, and optional timeout settings for the request to be sent.
///
/// # Returns
///
/// This function returns a `Result<HttpResponse, HttpError>`, which represents either the successful response from the server
/// (`HttpResponse`) or an error that occurred during the execution of the HTTP request (`HttpError`).
pub fn execute(request: impl Into<HttpRequest>) -> Result<HttpResponse, HttpError> {
    let request: HttpRequest = request.into();
    HttpClient::execute(request.0).map(Into::into).map_err(Into::into)
}

/// Executes multiple HTTP requests in a batch and returns their results.
///
/// This function takes advantage of `HttpClient::execute_many` to handle multiple requests concurrently
/// within the host runtime environment. Similar to executing single requests, this operation is blocking from
/// the guest's point of view but non-blocking on the host side where tasks can run asynchronously.
///
/// # Arguments
///
/// * `requests` - A `BatchHttpRequest` containing a vector of individual `crate::wit::HttpRequest`
///   objects. Each represents a complete HTTP request with its own settings and payload data to be sent.
///
/// # Returns
///
/// It returns a `Vec<Result<HttpResponse, HttpError>>`, which is a vector where each element corresponds
/// to the result of executing one of the batched requests. Each element will either contain an `HttpResponse`
/// if the request was successful or an `HttpError` if there was an issue with that particular request.
pub fn execute_many(requests: BatchHttpRequest) -> Vec<Result<HttpResponse, HttpError>> {
    HttpClient::execute_many(requests.requests)
        .into_iter()
        .map(|r| r.map(Into::into).map_err(Into::into))
        .collect()
}

impl From<http::Method> for HttpMethod {
    fn from(value: http::Method) -> Self {
        if value == http::Method::GET {
            Self::Get
        } else if value == http::Method::POST {
            Self::Post
        } else if value == http::Method::PUT {
            Self::Put
        } else if value == http::Method::DELETE {
            Self::Delete
        } else if value == http::Method::HEAD {
            Self::Head
        } else if value == http::Method::OPTIONS {
            Self::Options
        } else if value == http::Method::CONNECT {
            Self::Connect
        } else if value == http::Method::TRACE {
            Self::Trace
        } else if value == http::Method::PATCH {
            Self::Patch
        } else {
            unreachable!()
        }
    }
}

impl From<HttpMethod> for http::Method {
    fn from(value: HttpMethod) -> Self {
        match value {
            HttpMethod::Get => http::Method::GET,
            HttpMethod::Post => http::Method::POST,
            HttpMethod::Put => http::Method::PUT,
            HttpMethod::Delete => http::Method::DELETE,
            HttpMethod::Patch => http::Method::PATCH,
            HttpMethod::Head => http::Method::HEAD,
            HttpMethod::Options => http::Method::OPTIONS,
            HttpMethod::Connect => http::Method::CONNECT,
            HttpMethod::Trace => http::Method::TRACE,
        }
    }
}

/// HTTP error
pub enum HttpError {
    /// The request timed out.
    Timeout,
    /// The request could not be built correctly.
    Request(String),
    /// The request failed due to an error (server connection failed).
    Connect(String),
}

impl From<wit::HttpError> for HttpError {
    fn from(value: wit::HttpError) -> Self {
        match value {
            wit::HttpError::Timeout => Self::Timeout,
            wit::HttpError::Request(msg) => Self::Request(msg),
            wit::HttpError::Connect(msg) => Self::Connect(msg),
        }
    }
}

/// A struct that represents an HTTP request.
#[derive(Debug)]
pub struct HttpRequest(wit::HttpRequest);

impl HttpRequest {
    /// Constructs a new `HttpRequestBuilder` for sending a GET request to the specified URL.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL where the GET request should be sent.
    ///
    /// # Returns
    ///
    /// A builder object (`HttpRequestBuilder`) that can be used to further customize the HTTP request before execution.
    pub fn get(url: Url) -> HttpRequestBuilder {
        Self::builder(url, http::Method::GET)
    }

    /// Constructs a new `HttpRequestBuilder` for sending a POST request to the specified URL.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL where the POST request should be sent.
    ///
    /// # Returns
    ///
    /// A builder object (`HttpRequestBuilder`) that can be used to further customize the HTTP request before execution.
    pub fn post(url: Url) -> HttpRequestBuilder {
        Self::builder(url, http::Method::POST)
    }

    /// Constructs a new `HttpRequestBuilder` for sending a PUT request to the specified URL.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL where the PUT request should be sent.
    ///
    /// # Returns
    ///
    /// A builder object (`HttpRequestBuilder`) that can be used to further customize the HTTP request before execution.
    pub fn put(url: Url) -> HttpRequestBuilder {
        Self::builder(url, http::Method::PUT)
    }

    /// Constructs a new `HttpRequestBuilder` for sending a DELETE request to the specified URL.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL where the DELETE request should be sent.
    ///
    /// # Returns
    ///
    /// A builder object (`HttpRequestBuilder`) that can be used to further customize the HTTP request before execution.
    pub fn delete(url: Url) -> HttpRequestBuilder {
        Self::builder(url, http::Method::DELETE)
    }

    /// Constructs a new `HttpRequestBuilder` for sending a PATCH request to the specified URL.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL where the PATCH request should be sent.
    ///
    /// # Returns
    ///
    /// A builder object (`HttpRequestBuilder`) that can be used to further customize the HTTP request before execution.
    pub fn patch(url: Url) -> HttpRequestBuilder {
        Self::builder(url, http::Method::PATCH)
    }

    /// Constructs a new `HttpRequestBuilder` for sending a HEAD request to the specified URL.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL where the HEAD request should be sent.
    ///
    /// # Returns
    ///
    /// A builder object (`HttpRequestBuilder`) that can be used to further customize the HTTP request before execution.
    pub fn head(url: Url) -> HttpRequestBuilder {
        Self::builder(url, http::Method::HEAD)
    }

    /// Constructs a new `HttpRequestBuilder` for sending an OPTIONS request to the specified URL.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL where the OPTIONS request should be sent.
    ///
    /// # Returns
    ///
    /// A builder object (`HttpRequestBuilder`) that can be used to further customize the HTTP request before execution.
    pub fn options(url: Url) -> HttpRequestBuilder {
        Self::builder(url, http::Method::OPTIONS)
    }

    /// Constructs a new `HttpRequestBuilder` for sending a TRACE request to the specified URL.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL where the TRACE request should be sent.
    ///
    /// # Returns
    ///
    /// A builder object (`HttpRequestBuilder`) that can be used to further customize the HTTP request before execution.
    pub fn trace(url: Url) -> HttpRequestBuilder {
        Self::builder(url, http::Method::TRACE)
    }

    /// Constructs a new `HttpRequestBuilder` for sending a CONNECT request to the specified URL.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL where the CONNECT request should be sent.
    ///
    /// # Returns
    ///
    /// A builder object (`HttpRequestBuilder`) that can be used to further customize the HTTP request before execution.
    pub fn connect(url: Url) -> HttpRequestBuilder {
        Self::builder(url, http::Method::CONNECT)
    }

    /// Constructs a new `HttpRequestBuilder` for sending an HTTP request with the specified method and URL.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL where the request should be sent.
    /// * `method` - The HTTP method to use for the request (e.g., GET, POST).
    ///
    /// # Returns
    ///
    /// A builder object (`HttpRequestBuilder`) that can be used to further customize the HTTP request before execution.
    pub fn builder(url: Url, method: http::Method) -> HttpRequestBuilder {
        HttpRequestBuilder {
            method,
            url,
            headers: wit::Headers::new().into(),
            body: Default::default(),
            timeout: Default::default(),
        }
    }
}

/// A builder for constructing an `HttpRequest`.
pub struct HttpRequestBuilder {
    url: Url,
    method: http::Method,
    headers: Headers,
    body: Vec<u8>,
    timeout: Option<Duration>,
}

impl HttpRequestBuilder {
    /// Mutable access to the URL
    pub fn url(&mut self) -> &mut url::Url {
        &mut self.url
    }

    /// Mutable access to the HTTP headers of the request.
    pub fn headers(&mut self) -> &mut Headers {
        &mut self.headers
    }

    /// Adds a header to the HTTP request.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the header.
    /// * `value` - The value of the header.
    ///
    /// This method mutably modifies the builder, allowing headers to be added in sequence.
    pub fn header(&mut self, name: impl AsHeaderName, value: impl AsHeaderValue) -> &mut Self {
        self.headers.append(name, value);
        self
    }

    /// Sets a timeout for the HTTP request in milliseconds.
    ///
    /// # Arguments
    ///
    /// * `timeout_ms` - The duration of the timeout in milliseconds.
    ///
    /// This method mutably modifies the builder, setting an optional timeout for the request.
    pub fn timeout(&mut self, timeout: Duration) -> &mut Self {
        self.timeout = Some(timeout);
        self
    }

    /// Sets a JSON body for the HTTP request and adds the appropriate `Content-Type` header.
    ///
    /// # Type Parameters
    ///
    /// * `T` - A type that implements `Serialize`.
    ///
    /// # Arguments
    ///
    /// * `body` - The data to be serialized into JSON format and set as the body of the request.
    ///
    /// This method constructs a new `HttpRequest` with a JSON payload, returning it for execution.
    pub fn json<T: Serialize>(mut self, body: T) -> HttpRequest {
        self.headers.append("Content-Type", "application/json");

        self.body(serde_json::to_vec(&body).unwrap())
    }

    /// Sets a form-encoded body for the HTTP request and adds the appropriate `Content-Type` header.
    ///
    /// # Type Parameters
    ///
    /// * `T` - A type that implements `Serialize`.
    ///
    /// # Arguments
    ///
    /// * `body` - The data to be serialized into form-urlencoded format and set as the body of the request.
    ///
    /// This method constructs a new `HttpRequest` with a URL-encoded payload, returning it for execution.
    pub fn form<T: Serialize>(mut self, body: T) -> HttpRequest {
        self.headers.append("Content-Type", "application/x-www-form-urlencoded");

        self.body(serde_urlencoded::to_string(&body).unwrap().into_bytes())
    }

    /// Sets a raw byte array as the body for the HTTP request.
    ///
    /// # Arguments
    ///
    /// * `body` - The data to be set as the body of the request in `Vec<u8>` format.
    ///
    /// This method constructs and returns a new `HttpRequest` with the specified body.
    pub fn body(mut self, body: Vec<u8>) -> HttpRequest {
        self.body = body;
        self.build()
    }

    /// Constructs a fully configured `HttpRequest` from the builder.
    pub fn build(self) -> HttpRequest {
        HttpRequest(wit::HttpRequest {
            method: self.method.into(),
            url: self.url.to_string(),
            headers: self.headers.into(),
            body: self.body,
            timeout_ms: self.timeout.map(|d| d.as_millis() as u64),
        })
    }
}

impl From<HttpRequestBuilder> for HttpRequest {
    fn from(builder: HttpRequestBuilder) -> Self {
        builder.build()
    }
}

/// A structure representing a batch of HTTP requests.
pub struct BatchHttpRequest {
    /// A vector holding individual `crate::wit::HttpRequest` objects that are part of this batch.
    pub(crate) requests: Vec<wit::HttpRequest>,
}

impl BatchHttpRequest {
    /// Constructs a new, empty `BatchHttpRequest`.
    pub fn new() -> Self {
        Self { requests: Vec::new() }
    }

    /// Adds a single HTTP request to the batch.
    pub fn push(&mut self, request: HttpRequest) {
        self.requests.push(request.0);
    }

    /// Returns the number of HTTP requests in the batch.
    pub fn len(&self) -> usize {
        self.requests.len()
    }

    /// Determines whether the batch of HTTP requests is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for BatchHttpRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// A struct that represents an HTTP response.
pub struct HttpResponse {
    status_code: http::StatusCode,
    headers: Headers,
    body: Vec<u8>,
}

impl From<wit::HttpResponse> for HttpResponse {
    fn from(response: wit::HttpResponse) -> Self {
        Self {
            status_code: http::StatusCode::from_u16(response.status).expect("Provided by the host"),
            headers: response.headers.into(),
            body: response.body,
        }
    }
}

impl HttpResponse {
    /// Returns the status code of the HTTP response.
    pub fn status(&self) -> http::StatusCode {
        self.status_code
    }

    /// Returns the headers of the HTTP response.
    pub fn headers(&self) -> &Headers {
        &self.headers
    }

    /// Returns the body of the HTTP response.
    pub fn body(&self) -> &[u8] {
        &self.body
    }

    /// Converts the HTTP response body into a `Vec<u8>`.
    pub fn into_bytes(self) -> Vec<u8> {
        self.body
    }

    /// Attempts to convert the HTTP response body into a UTF-8 encoded `String`.
    ///
    /// This method takes ownership of the `HttpResponse` and returns a `Result<String, std::string::FromUtf8Error>`.
    /// It attempts to interpret the bytes in the body as a valid UTF-8 sequence.
    pub fn text(self) -> Result<String, FromUtf8Error> {
        String::from_utf8(self.body)
    }

    /// Attempts to deserialize the HTTP response body as JSON.
    ///
    /// This method takes ownership of the `HttpResponse` and returns a `Result<serde_json::Value, serde_json::Error>`.
    ///
    /// It attempts to interpret the bytes in the body as valid JSON. The conversion is successful if the
    /// byte slice represents a valid JSON value according to the JSON specification.
    pub fn json<'de, T>(&'de self) -> Result<T, JsonDeserializeError>
    where
        T: serde::de::Deserialize<'de>,
    {
        serde_json::from_slice(&self.body)
    }
}
