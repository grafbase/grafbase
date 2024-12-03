//! A HTTP client module for executing HTTP requests in the host runtime.
//! The functions are blocking from the guest, but are run asynchronously in the host.
//! While the request is being executed, the host thread can continue to run other code.

use crate::wit::HttpClient;
pub use crate::wit::{HttpError, HttpMethod, HttpRequest, HttpResponse, HttpVersion};

/// Executes a single HTTP request in the host runtime.
pub fn execute(request: &HttpRequest) -> Result<HttpResponse, HttpError> {
    HttpClient::execute(request)
}

/// Executes multiple HTTP requests in the host runtime in parallel.
/// Returns results in the same order as the requests.
pub fn execute_many(requests: &[HttpRequest]) -> Vec<Result<HttpResponse, HttpError>> {
    HttpClient::execute_many(requests)
}
