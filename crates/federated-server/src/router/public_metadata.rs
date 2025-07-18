use axum::body::Bytes;
use futures_util::future::BoxFuture;

/// Creates a handler for public metadata endpoints that returns a pre-configured response.
pub(super) fn handler(
    response_body: Bytes,
    headers: http::HeaderMap,
) -> impl Fn() -> BoxFuture<'static, axum::response::Response> + Clone {
    move || {
        let headers = headers.clone();
        let response_body = response_body.clone();
        Box::pin(async move {
            let mut response = axum::response::Response::new(axum::body::Body::from(response_body));

            *response.headers_mut() = headers;

            response
        })
    }
}
