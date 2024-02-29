use axum::{
    body::Body,
    extract::Request,
    middleware::{self, Next},
    response::Response,
    Router,
};

const GRAFBASE_CSRF_HEADER: &str = "X-Grafbase-CSRF-Protection";

pub(super) fn inject_layer(mut router: Router) -> Router {
    router = router.layer(middleware::from_fn(csrf_middleware));
    router
}

async fn csrf_middleware(request: Request, next: Next) -> Response {
    if validates_csrf(&request) {
        return next.run(request).await;
    }

    Response::builder()
        .status(http::StatusCode::FORBIDDEN)
        .body(Body::empty())
        .expect("cannot fail")
}

fn validates_csrf(request: &Request) -> bool {
    request.method() == http::Method::OPTIONS || request.headers().contains_key(GRAFBASE_CSRF_HEADER)
}
