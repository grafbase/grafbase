use std::{fmt::Display, future::Future, pin::Pin};

use http::{Request, Response};
use http_body::Body;
use runtime::extension::HooksExtension;
use tower::Layer;

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

impl<Service, Hooks, ReqBody, ResBody> tower::Service<Request<ReqBody>> for HooksService<Service, Hooks>
where
    Service: tower::Service<Request<ReqBody>, Response = Response<ResBody>> + Send + Clone + 'static,
    Service::Future: Send,
    Hooks: HooksExtension + Clone,
    Service::Error: Display + 'static,
    ReqBody: Body + Send + 'static,
    ResBody: Body + Send + Default + From<Vec<u8>> + 'static,
{
    type Response = http::Response<ResBody>;
    type Error = Service::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Response<ResBody>, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let mut inner = self.inner.clone();
        let hooks = self.hooks.clone();
        let context = hooks.new_context();

        Box::pin(async move {
            let (parts, body) = req.into_parts();

            let response_format = crate::error_response::extract_response_format(&parts.headers);

            let parts = match hooks.on_request(&context, parts).await {
                Ok(parts) => parts,
                Err(err) => {
                    let error_response = crate::error_response::request_error_response_to_http(response_format, err);
                    return Ok(error_response);
                }
            };

            let mut request = Request::from_parts(parts, body);
            request.extensions_mut().insert(context.clone());

            let response = match inner.call(request).await {
                Ok(response) => response,
                Err(err) => {
                    return Err(err);
                }
            };

            let (parts, body) = response.into_parts();

            let parts = match hooks.on_response(&context, parts).await {
                Ok(parts) => parts,
                Err(err) => {
                    tracing::error!("Error in on_response hook: {err}");

                    let error_response = crate::error_response::response_error_response_to_http(response_format, err);

                    return Ok(error_response);
                }
            };

            let response = Response::from_parts(parts, body);

            Ok(response)
        })
    }
}
