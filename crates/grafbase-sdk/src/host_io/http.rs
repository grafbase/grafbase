//! A module for executing HTTP requests using a client that operates within the host runtime.
//! Functions are synchronous from the guest's perspective, yet they run asynchronously on the host side.
//! While waiting for request completion, the host thread can execute other tasks concurrently.

use serde::Serialize;
pub use url::Url;

use crate::wit::HttpClient;
pub use crate::wit::{HttpError, HttpMethod, HttpResponse, HttpVersion};

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
    HttpClient::execute(&request.0)
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
}

impl HttpMethod {
    /// Returns string slice representation of HTTP Method.
    pub fn as_str(&self) -> &str {
        match self {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Delete => "DELETE",
            HttpMethod::Patch => "PATCH",
            HttpMethod::Head => "HEAD",
            HttpMethod::Options => "OPTIONS",
            HttpMethod::Trace => "TRACE",
            HttpMethod::Connect => "CONNECT",
        }
    }
}

/// A struct that represents an HTTP request.
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
        Self::builder(url, HttpMethod::Get)
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
        Self::builder(url, HttpMethod::Post)
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
        Self::builder(url, HttpMethod::Put)
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
        Self::builder(url, HttpMethod::Delete)
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
        Self::builder(url, HttpMethod::Patch)
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
        Self::builder(url, HttpMethod::Head)
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
        Self::builder(url, HttpMethod::Options)
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
        Self::builder(url, HttpMethod::Trace)
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
        Self::builder(url, HttpMethod::Connect)
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
    pub fn builder(url: Url, method: HttpMethod) -> HttpRequestBuilder {
        HttpRequestBuilder(crate::wit::HttpRequest {
            method,
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
    pub fn json<T: Serialize + ?Sized>(mut self, body: &T) -> HttpRequest {
        self.0
            .headers
            .push(("Content-Type".to_string(), "application/json".to_string()));

        self.body(serde_json::to_vec(body).unwrap())
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
    pub fn form<T: Serialize + ?Sized>(mut self, body: &T) -> HttpRequest {
        self.0.headers.push((
            "Content-Type".to_string(),
            "application/x-www-form-urlencoded".to_string(),
        ));

        self.body(serde_urlencoded::to_string(body).unwrap().into_bytes())
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
