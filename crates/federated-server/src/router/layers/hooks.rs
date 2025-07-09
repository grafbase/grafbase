use std::{fmt::Display, future::Future, pin::Pin};

use event_queue::ExecutedHttpRequest;
use http::{Request, Response};
use http_body::Body;
use runtime::extension::{ExtensionContext, HooksExtension};
use tower::Layer;

use crate::engine::into_axum_response;

#[derive(Clone)]
pub struct HooksLayer<Hooks> {
    hooks: Hooks,
}

impl<Hooks> HooksLayer<Hooks>
where
    Hooks: HooksExtension,
{
    pub fn new(hooks: Hooks) -> Self {
        Self { hooks }
    }
}

impl<Service, Hooks> Layer<Service> for HooksLayer<Hooks>
where
    Hooks: HooksExtension + Clone,
    Service: Send + Clone,
{
    type Service = HooksService<Service, Hooks>;

    fn layer(&self, inner: Service) -> Self::Service {
        HooksService {
            inner,
            hooks: self.hooks.clone(),
        }
    }
}

#[derive(Clone)]
pub struct HooksService<Service, Hooks> {
    inner: Service,
    hooks: Hooks,
}

impl<Service, Hooks, ReqBody> tower::Service<Request<ReqBody>> for HooksService<Service, Hooks>
where
    Service: tower::Service<Request<ReqBody>, Response = Response<axum::body::Body>> + Send + Clone + 'static,
    Service::Future: Send,
    Hooks: HooksExtension + Clone,
    Service::Error: Display + 'static,
    ReqBody: Body + Send + 'static,
{
    type Response = http::Response<axum::body::Body>;
    type Error = Service::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Response<axum::body::Body>, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let mut inner = self.inner.clone();
        let hooks = self.hooks.clone();
        let context = hooks.new_context();

        Box::pin(async move {
            let (parts, body) = req.into_parts();
            let url = parts.uri.to_string();
            let method = parts.method.clone();

            let response_format = engine::ResponseFormat::extract_from(&parts.headers).unwrap_or_default();

            let parts = match hooks.on_request(&context, parts).await {
                Ok(parts) => parts,
                Err(err) => {
                    let error_response = engine::http_error_response(response_format, err);
                    return Ok(into_axum_response(error_response));
                }
            };

            let mut request = Request::from_parts(parts, body);
            request.extensions_mut().insert(context.clone());

            let response = inner.call(request).await?;

            let (parts, body) = response.into_parts();

            let builder = ExecutedHttpRequest::builder(&url)
                .method(method)
                .response_status(parts.status);

            context.event_queue().push_http_request(builder);

            let parts = match hooks.on_response(&context, parts).await {
                Ok(parts) => parts,
                Err(err) => {
                    let error_response = engine::http_error_response(
                        response_format,
                        engine::ErrorResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                            .with_error(engine::GraphqlError::new(err, engine::ErrorCode::ExtensionError)),
                    );

                    return Ok(into_axum_response(error_response));
                }
            };

            let response = Response::from_parts(parts, body);

            Ok(response)
        })
    }
}
