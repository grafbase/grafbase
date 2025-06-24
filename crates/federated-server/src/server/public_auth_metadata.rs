use axum::body::Bytes;
use std::pin::Pin;

pub(super) fn public_metadata_handler(
    response_body: Bytes,
    headers: http::HeaderMap,
) -> impl FnOnce() -> Pin<Box<dyn Future<Output = axum::response::Response> + Send + Sync + 'static>> + Clone {
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
