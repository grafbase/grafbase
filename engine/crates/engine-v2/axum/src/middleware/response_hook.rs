use futures_util::Future;
use http::{response, Request, Response};
use http_body::Body;
use runtime::hooks::{self, ExecutedHttpRequest, ResponseHooks};
use std::{fmt::Display, pin::Pin};
use tower::Layer;

#[derive(Clone)]
pub struct ResponseHookLayer<Hooks> {
    hooks: Hooks,
}

impl<Hooks> ResponseHookLayer<Hooks>
where
    Hooks: hooks::Hooks + Clone,
{
    pub fn new(hooks: Hooks) -> Self {
        Self { hooks }
    }
}

impl<Service, Hooks> Layer<Service> for ResponseHookLayer<Hooks>
where
    Hooks: hooks::Hooks + Clone,
    Service: Send + Clone,
{
    type Service = ResponseHookService<Service, Hooks>;

    fn layer(&self, inner: Service) -> Self::Service {
        ResponseHookService {
            inner,
            hooks: self.hooks.clone(),
        }
    }
}

#[derive(Clone)]
pub struct ResponseHookService<Service, Hooks> {
    inner: Service,
    hooks: Hooks,
}

impl<Service, Hooks, ReqBody, ResBody> tower::Service<Request<ReqBody>> for ResponseHookService<Service, Hooks>
where
    Service: tower::Service<Request<ReqBody>, Response = Response<ResBody>> + Send + Clone + 'static,
    Service::Future: Send,
    Hooks: hooks::Hooks + Clone,
    Service::Error: Display + 'static,
    ReqBody: Body + Send + 'static,
    ResBody: Body + Send + Default + 'static,
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
        let method = req.method().clone();
        let url = req.uri().clone();

        Box::pin(async move {
            let mut response = match inner.call(req).await {
                Ok(response) => response,
                Err(e) => return Err(e),
            };

            let Some((ctx, on_operation_response_outputs)) =
                response.extensions_mut().remove::<(Hooks::Context, Vec<Vec<u8>>)>()
            else {
                return Ok(response);
            };

            let request_info = ExecutedHttpRequest {
                method,
                url,
                status_code: response.status(),
                on_operation_response_outputs,
            };

            let response = match hooks.responses().on_http_response(&ctx, request_info).await {
                Ok(_) => response,
                Err(e) => {
                    tracing::error!("error calling on-http-response hook: {e}");

                    response::Builder::new()
                        .status(500)
                        .body(Default::default())
                        .expect("cannot fail")
                }
            };

            Ok(response)
        })
    }
}
