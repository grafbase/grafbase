//! This module defines a simple cross site request forgery prevention mechanism described in:
//! https://cheatsheetseries.owasp.org/cheatsheets/Cross-Site_Request_Forgery_Prevention_Cheat_Sheet.html#cross-site-request-forgery-prevention-cheat-sheet
//!
//! The basic idea is the following:
//!
//! In general we want CORS to define from which browsers you can connect to the backend (this service).
//! This is handled by pre-flight requests in every modern browser: `OPTIONS` first then `POST` or `GET` etc.
//! Now, there's one case where the browser doesn't send pre-flight requests, and it's with a simple `GET`. This
//! can be an attack vector, so the prevention mechanism here is to require a custom header to be present. A
//! simple request in the browser cannot change the headers, so this is enough to prevent the attack vector.

use std::sync::Arc;

use axum::{
    Router,
    body::Body,
    extract::{Request, State},
    middleware::{self, Next},
    response::Response,
};
use gateway_config::CsrfConfig;

/// Injects a middleware layer into the given router for CSRF protection.
///
/// This middleware checks for the presence of a custom CSRF header in incoming requests
/// to prevent Cross-Site Request Forgery (CSRF) attacks. If the header is missing and the
/// request is not a pre-flight `OPTIONS` request, the middleware responds with a 403 Forbidden
/// status.
pub(super) fn inject_layer(mut router: Router, config: &CsrfConfig) -> Router {
    assert!(config.enabled);
    router = router.layer(middleware::from_fn_with_state(
        Arc::new(config.clone()),
        csrf_middleware,
    ));
    router
}

async fn csrf_middleware(State(config): State<Arc<CsrfConfig>>, request: Request, next: Next) -> Response {
    let valid_csrf = request.method() == http::Method::OPTIONS || request.headers().contains_key(&config.header_name);
    if valid_csrf {
        return next.run(request).await;
    }

    Response::builder()
        .status(http::StatusCode::FORBIDDEN)
        .body(Body::empty())
        .expect("cannot fail")
}
