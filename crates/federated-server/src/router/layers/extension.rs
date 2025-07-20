use std::{fmt::Display, future::Future, pin::Pin};

use axum::body::Body;
use engine::RequestExtensions;
use event_queue::ExecutedHttpRequest;
use http::{Request, Response};
use runtime::{
    authentication::Authenticate,
    extension::{ExtensionContext, GatewayExtensions, OnRequest},
};
use tower::Layer;

use crate::engine::into_axum_response;

#[derive(Clone)]
pub struct ExtensionLayer<Ext, A> {
    extensions: Ext,
    auth: A,
}

impl<Ext, A> ExtensionLayer<Ext, A>
where
    Ext: GatewayExtensions,
    A: Authenticate<<Ext as GatewayExtensions>::Context>,
{
    pub fn new(extensions: Ext, auth: A) -> Self {
        Self { extensions, auth }
    }
}

impl<Service, Ext, A> Layer<Service> for ExtensionLayer<Ext, A>
where
    Ext: GatewayExtensions,
    A: Authenticate<<Ext as GatewayExtensions>::Context>,
    Service: Send + Clone,
{
    type Service = ExtensionService<Service, Ext, A>;

    fn layer(&self, inner: Service) -> Self::Service {
        ExtensionService {
            inner,
            extensions: self.extensions.clone(),
            auth: self.auth.clone(),
        }
    }
}

#[derive(Clone)]
pub struct ExtensionService<Service, Ext, A> {
    inner: Service,
    extensions: Ext,
    auth: A,
}

impl<Service, Ext, A, ReqBody> tower::Service<Request<ReqBody>> for ExtensionService<Service, Ext, A>
where
    Service: tower::Service<Request<ReqBody>, Response = Response<Body>> + Send + Clone + 'static,
    Service::Future: Send,
    Service::Error: Display + 'static,
    ReqBody: http_body::Body + Send + 'static,
    Ext: GatewayExtensions,
    A: Authenticate<<Ext as GatewayExtensions>::Context>,
{
    type Response = http::Response<Body>;
    type Error = Service::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Response<Body>, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let mut inner = self.inner.clone();
        let extensions = self.extensions.clone();
        let auth = self.auth.clone();

        Box::pin(async move {
            let (parts, body) = req.into_parts();
            let url = parts.uri.to_string();
            let method = parts.method.clone();

            let response_format = engine::ResponseFormat::extract_from(&parts.headers).unwrap_or_default();

            let OnRequest {
                context,
                mut parts,
                contract_key,
            } = match extensions.on_request(parts).await {
                Ok(on_request) => on_request,
                Err(err) => {
                    let error_response = engine::http_error_response(response_format, err);
                    return Ok(into_axum_response(error_response));
                }
            };

            let headers = std::mem::take(&mut parts.headers);
            let response = match auth.authenticate(&context, headers).await {
                Ok((headers, token)) => {
                    parts.headers = headers;
                    parts
                        .extensions
                        .insert(RequestExtensions::<<Ext as GatewayExtensions>::Context> {
                            context: context.clone(),
                            token,
                            contract_key,
                        });

                    inner.call(Request::from_parts(parts, body)).await?
                }
                Err(err) => {
                    let error_response = engine::http_error_response(response_format, err);
                    into_axum_response(error_response)
                }
            };

            let (parts, body) = response.into_parts();

            let builder = ExecutedHttpRequest::builder(&url)
                .method(method)
                .response_status(parts.status);

            context.event_queue().push_http_request(builder);

            let parts = match extensions.on_response(context, parts).await {
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
