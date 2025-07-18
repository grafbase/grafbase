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

use std::{fmt::Display, pin::Pin, sync::Arc};

use axum::{
    extract::Request,
    response::{IntoResponse, Response},
};
use gateway_config::CsrfConfig;

#[derive(Clone)]
pub struct CsrfLayer {
    config: Arc<CsrfConfig>,
}

/// This layer checks for the presence of a custom CSRF header in incoming requests
/// to prevent Cross-Site Request Forgery (CSRF) attacks. If the header is missing and the
/// request is not a pre-flight `OPTIONS` request, the middleware responds with a 403 Forbidden
/// status.
impl CsrfLayer {
    pub fn new(config: &CsrfConfig) -> Self {
        Self {
            config: Arc::new(config.clone()),
        }
    }
}

impl<Service> tower::Layer<Service> for CsrfLayer
where
    Service: Send + Clone,
{
    type Service = CsrfService<Service>;

    fn layer(&self, inner: Service) -> Self::Service {
        CsrfService {
            inner,
            config: self.config.clone(),
        }
    }
}

#[derive(Clone)]
pub struct CsrfService<S> {
    inner: S,
    config: Arc<CsrfConfig>,
}

impl<Service, ReqBody> tower::Service<Request<ReqBody>> for CsrfService<Service>
where
    Service: tower::Service<Request<ReqBody>> + Send + Clone + 'static,
    Service::Future: Send,
    Service::Error: Display + 'static,
    Service::Response: IntoResponse,
    ReqBody: http_body::Body + Send + 'static,
{
    type Response = axum::response::Response;
    type Error = Service::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request<ReqBody>) -> Self::Future {
        let mut inner = self.inner.clone();
        let config = self.config.clone();

        Box::pin(async move {
            let valid_csrf = !config.enabled
                || request.method() == http::Method::OPTIONS
                || request.headers().contains_key(&config.header_name);
            if valid_csrf {
                return inner.call(request).await.map(|r| r.into_response());
            }

            Ok(Response::builder()
                .status(http::StatusCode::FORBIDDEN)
                .body(axum::body::Body::empty())
                .expect("cannot fail"))
        })
    }
}
