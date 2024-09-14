use engine_v2::HooksExtension;
use futures_util::{Future, StreamExt};
use http::{response, Request, Response};
use http_body::Body;
use runtime::hooks::{self, ExecutedHttpRequest, ResponseHooks};
use std::{fmt::Display, pin::Pin};
use tower::Layer;

#[derive(Clone)]
/// A layer that adds calling the `on-http-response` hook to a service.
pub struct ResponseHookLayer<Hooks> {
    hooks: Hooks,
}

impl<Hooks> ResponseHookLayer<Hooks>
where
    Hooks: hooks::Hooks + Clone,
{
    /// Creates a new `ResponseHookLayer` with the provided hooks.
    ///
    /// # Arguments
    ///
    /// * `hooks` - An instance of the hooks to be used by this layer.
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
/// A service that handles HTTP requests and applies the `on-http-response` hook.
///
/// The `ResponseHookService` wraps a service and invokes the provided hook
/// when it receives an HTTP response. It can be used to add custom behavior
/// after an HTTP response is generated, such as logging or modifying the
/// response before it is sent to the client.
///
/// # Type Parameters
///
/// * `Service` - The type of the inner service being wrapped.
/// * `Hooks` - The type of hooks that will be called upon receiving the response.
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

    /// Calls the inner service with the provided HTTP request, invoking the
    /// `on-http-response` hook upon receiving the response.
    ///
    /// # Arguments
    ///
    /// * `req` - The HTTP request to be processed by the inner service.
    ///
    /// # Returns
    ///
    /// This method returns a future that resolves to either the HTTP response
    /// returned by the inner service or an error if the service fails to process
    /// the request.
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

            let Some(extension) = response.extensions_mut().remove::<HooksExtension<Hooks::Context>>() else {
                return Ok(response);
            };

            let (context, on_operation_response_outputs) = match extension {
                HooksExtension::Single {
                    context,
                    on_operation_response_output,
                } => (context, on_operation_response_output.into_iter().collect()),
                HooksExtension::Batch {
                    context,
                    on_operation_response_outputs,
                } => (context, on_operation_response_outputs),
                HooksExtension::Stream {
                    context,
                    mut on_operation_response_outputs,
                } => (
                    context,
                    // TODO: Currently we only handle query/mutations which return the complete
                    // response at once and errors.
                    on_operation_response_outputs
                        .next()
                        .await
                        .map(|out| vec![out])
                        .unwrap_or_default(),
                ),
            };

            let request_info = ExecutedHttpRequest {
                method,
                url,
                status_code: response.status(),
                on_operation_response_outputs,
            };

            let response = match hooks.responses().on_http_response(&context, request_info).await {
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
