//! A module for executing HTTP requests.

use std::string::FromUtf8Error;

pub use crate::wit::{HttpError, HttpMethod, HttpVersion};
pub use http::{HeaderName, HeaderValue, Method, StatusCode};
pub use serde_json::Error as JsonDeserializeError;
pub use url::Url;

use crate::wit::HttpClient;
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
pub fn execute(request: &HttpRequest) -> Result<HttpResponse, HttpError> {
    HttpClient::execute(&request.0).map(HttpResponse)
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
    HttpClient::execute_many(&requests.requests)
        .into_iter()
        .map(|r| r.map(HttpResponse))
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

/// A struct that represents an HTTP request.
#[derive(Debug)]
pub struct HttpRequest(crate::wit::HttpRequest);

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
        HttpRequestBuilder(crate::wit::HttpRequest {
            method: method.into(),
            url: url.to_string(),
            headers: Default::default(),
            body: Default::default(),
            timeout_ms: Default::default(),
        })
    }
}

/// A builder for constructing an `HttpRequest`.
pub struct HttpRequestBuilder(crate::wit::HttpRequest);

impl HttpRequestBuilder {
    /// Sets the URI for the HTTP request.
    ///
    /// # Arguments
    ///
    /// * `uri` - The new URL to which the request will be sent.
    ///
    /// # Returns
    ///
    /// A mutable reference to self, allowing further chaining of builder methods.
    pub fn uri(mut self, uri: Url) -> Self {
        self.0.url = uri.to_string();
        self
    }

    /// Adds a header to the HTTP request.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the header.
    /// * `value` - The value of the header.
    ///
    /// This method mutably modifies the builder, allowing headers to be added in sequence.
    pub fn push_header(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.0.headers.push((name.into(), value.into()));
    }

    /// Sets a timeout for the HTTP request in milliseconds.
    ///
    /// # Arguments
    ///
    /// * `timeout_ms` - The duration of the timeout in milliseconds.
    ///
    /// This method mutably modifies the builder, setting an optional timeout for the request.
    pub fn set_timeout(&mut self, timeout_ms: u64) {
        self.0.timeout_ms = Some(timeout_ms);
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
        self.0
            .headers
            .push(("Content-Type".to_string(), "application/json".to_string()));

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
        self.0.headers.push((
            "Content-Type".to_string(),
            "application/x-www-form-urlencoded".to_string(),
        ));

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
        self.0.body = body;
        self.build()
    }

    /// Constructs a fully configured `HttpRequest` from the builder.
    pub fn build(self) -> HttpRequest {
        HttpRequest(self.0)
    }
}

/// A structure representing a batch of HTTP requests.
pub struct BatchHttpRequest {
    /// A vector holding individual `crate::wit::HttpRequest` objects that are part of this batch.
    pub(crate) requests: Vec<crate::wit::HttpRequest>,
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
pub struct HttpResponse(crate::wit::HttpResponse);

impl HttpResponse {
    /// Returns the status code of the HTTP response.
    pub fn status(&self) -> http::StatusCode {
        http::StatusCode::from_u16(self.0.status).expect("must be valid, this comes from reqwest")
    }

    /// Returns the headers of the HTTP response.
    pub fn headers(&self) -> &[(String, String)] {
        &self.0.headers
    }

    /// Returns the body of the HTTP response.
    pub fn body(&self) -> &[u8] {
        &self.0.body
    }

    /// Converts the HTTP response body into a `Vec<u8>`.
    pub fn into_bytes(self) -> Vec<u8> {
        self.0.body
    }

    /// Attempts to convert the HTTP response body into a UTF-8 encoded `String`.
    ///
    /// This method takes ownership of the `HttpResponse` and returns a `Result<String, std::string::FromUtf8Error>`.
    /// It attempts to interpret the bytes in the body as a valid UTF-8 sequence.
    pub fn text(self) -> Result<String, FromUtf8Error> {
        String::from_utf8(self.0.body)
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
        serde_json::from_slice(&self.0.body)
    }
}
