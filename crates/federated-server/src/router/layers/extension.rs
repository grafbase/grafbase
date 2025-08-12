use std::{fmt::Display, future::Future, pin::Pin, sync::Arc};

use axum::body::Body;
use engine::{ErrorResponse, GraphqlError, RequestExtensions};
use event_queue::ExecutedHttpRequest;
use extension_catalog::ExtensionId;
use gateway_config::DefaultAuthenticationBehavior;
use http::{Request, Response};
use runtime::extension::{ExtensionContext, GatewayExtensions, OnRequest, Token};
use tower::Layer;

use crate::engine::into_axum_response;

#[derive(Clone)]
pub struct ExtensionLayer<Ext>(Arc<ExtensionLayerInner<Ext>>);

struct ExtensionLayerInner<Ext> {
    extensions: Ext,
    default_contract_key: Option<String>,
    authentication_extension_ids: Vec<ExtensionId>,
    default_authentication_behavior: Option<DefaultAuthenticationBehavior>,
}

impl<Ext> ExtensionLayer<Ext>
where
    Ext: GatewayExtensions,
{
    pub fn new(
        extensions: Ext,
        default_contract_key: Option<String>,
        authentication_extension_ids: Vec<ExtensionId>,
        default_authentication_behavior: Option<DefaultAuthenticationBehavior>,
    ) -> Self {
        Self(Arc::new(ExtensionLayerInner {
            extensions,
            default_contract_key,
            authentication_extension_ids,
            default_authentication_behavior,
        }))
    }
}

impl<Service, Ext> Layer<Service> for ExtensionLayer<Ext>
where
    Ext: GatewayExtensions,
    Service: Send + Clone,
{
    type Service = ExtensionService<Service, Ext>;

    fn layer(&self, next: Service) -> Self::Service {
        ExtensionService {
            next,
            layer: self.0.clone(),
        }
    }
}

#[derive(Clone)]
pub struct ExtensionService<Service, Ext> {
    next: Service,
    layer: Arc<ExtensionLayerInner<Ext>>,
}

impl<Service, Ext, ReqBody> tower::Service<Request<ReqBody>> for ExtensionService<Service, Ext>
where
    Service: tower::Service<Request<ReqBody>, Response = Response<Body>> + Send + Clone + 'static,
    Service::Future: Send,
    Service::Error: Display + 'static,
    ReqBody: http_body::Body + Send + 'static,
    Ext: GatewayExtensions,
{
    type Response = http::Response<Body>;
    type Error = Service::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Response<Body>, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.next.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let mut next = self.next.clone();
        let layer = self.layer.clone();

        Box::pin(async move {
            let (parts, body) = req.into_parts();
            let url = parts.uri.to_string();
            let method = parts.method.clone();

            let response_format = engine::ResponseFormat::extract_from(&parts.headers).unwrap_or_default();

            let OnRequest {
                context,
                mut parts,
                contract_key,
                context: state,
            } = match layer.extensions.on_request(parts).await {
                Ok(on_request) => on_request,
                Err(err) => {
                    let error_response = engine::http_error_response(response_format, err);
                    return Ok(into_axum_response(error_response));
                }
            };

            let result = if layer.authentication_extension_ids.is_empty() {
                match layer.default_authentication_behavior {
                    Some(DefaultAuthenticationBehavior::Anonymous) | None => Ok(Token::Anonymous),
                    Some(DefaultAuthenticationBehavior::Deny) => {
                        Err(ErrorResponse::new(http::StatusCode::UNAUTHORIZED)
                            .with_error(GraphqlError::unauthenticated()))
                    }
                }
            } else {
                let headers = std::mem::take(&mut parts.headers);
                let (headers, result) = layer
                    .extensions
                    .authenticate(&context, headers, &layer.authentication_extension_ids)
                    .await;
                parts.headers = headers;
                match result {
                    Ok(token) => Ok(token),
                    Err(err) => match layer.default_authentication_behavior {
                        Some(DefaultAuthenticationBehavior::Anonymous) => Ok(Token::Anonymous),
                        Some(DefaultAuthenticationBehavior::Deny) | None => Err(err),
                    },
                }
            };

            let response = match result {
                Ok(token) => {
                    parts
                        .extensions
                        .insert(RequestExtensions::<<Ext as GatewayExtensions>::Context> {
                            context: context.clone(),
                            token,
                            contract_key: contract_key.or_else(|| layer.default_contract_key.clone()),
                        });

                    next.call(Request::from_parts(parts, body)).await?
                }
                Err(err) => {
                    let error_response = engine::http_error_response(response_format, err);
                    return Ok(into_axum_response(error_response));
                }
            };

            let (parts, body) = response.into_parts();

            let builder = ExecutedHttpRequest::builder(&url)
                .method(method)
                .response_status(parts.status);

            context.event_queue().push_http_request(builder);

            let parts = match layer.extensions.on_response(context, parts).await {
                Ok(parts) => parts,
                Err(err) => {
                    let error_response = engine::http_error_response(
                        response_format,
                        ErrorResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
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
