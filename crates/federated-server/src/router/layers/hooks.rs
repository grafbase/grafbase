use std::{fmt::Display, future::Future, pin::Pin};

use event_queue::ExecutedHttpRequest;
use http::{Request, Response};
use http_body::Body;
use runtime::extension::{ExtensionContext, GatewayExtensions};
use tower::Layer;

use crate::engine::into_axum_response;

#[derive(Clone)]
pub struct HooksLayer<Ext> {
    extensions: Ext,
}

impl<Ext> HooksLayer<Ext>
where
    Ext: GatewayExtensions,
{
    pub fn new(extensions: Ext) -> Self {
        Self { extensions }
    }
}

impl<Service, Ext> Layer<Service> for HooksLayer<Ext>
where
    Ext: GatewayExtensions + Clone,
    Service: Send + Clone,
{
    type Service = HooksService<Service, Ext>;

    fn layer(&self, inner: Service) -> Self::Service {
        HooksService {
            inner,
            extensions: self.extensions.clone(),
        }
    }
}

#[derive(Clone)]
pub struct HooksService<Service, Ext> {
    inner: Service,
    extensions: Ext,
}

impl<Service, Ext, ReqBody> tower::Service<Request<ReqBody>> for HooksService<Service, Ext>
where
    Service: tower::Service<Request<ReqBody>, Response = Response<axum::body::Body>> + Send + Clone + 'static,
    Service::Future: Send,
    Service::Error: Display + 'static,
    ReqBody: Body + Send + 'static,
    Ext: GatewayExtensions,
{
    type Response = http::Response<axum::body::Body>;
    type Error = Service::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Response<axum::body::Body>, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let mut inner = self.inner.clone();
        let extensions = self.extensions.clone();

        Box::pin(async move {
            let (parts, body) = req.into_parts();
            let url = parts.uri.to_string();
            let method = parts.method.clone();

            let response_format = engine::ResponseFormat::extract_from(&parts.headers).unwrap_or_default();

            let (context, parts) = match extensions.on_request(parts).await {
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

            let parts = match extensions.on_response(&context, parts).await {
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
