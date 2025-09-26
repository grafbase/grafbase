use std::sync::Arc;

use error::{ErrorCode, GraphqlError};
use futures::StreamExt;
use operation::{BatchRequest, Request};

use crate::{
    Body, Engine,
    graphql_over_http::{Http, ResponseFormat},
    response::Response,
};

use super::{RequestContext, Runtime, response_extension::default_response_extensions, stream::StreamResponse};

impl<R: Runtime> Engine<R> {
    pub(crate) async fn execute_well_formed_graphql_request(
        self: &Arc<Self>,
        request_context: Arc<RequestContext>,
        request: BatchRequest,
    ) -> http::Response<Body> {
        match request {
            BatchRequest::Single(request) => match request_context.response_format {
                ResponseFormat::Streaming(format) => {
                    Http::stream(format, self.execute_stream(request_context, request)).await
                }
                ResponseFormat::Complete(format) => {
                    let Some(response) = self
                        .with_gateway_timeout(self.execute_single(&request_context, request))
                        .await
                    else {
                        return self.gateway_timeout_error(&request_context);
                    };

                    Http::single(format, response)
                }
            },
            BatchRequest::Batch(requests) => {
                let ResponseFormat::Complete(format) = request_context.response_format else {
                    return self.bad_request_but_well_formed_graphql_over_http_request(
                        &request_context,
                        "batch requests cannot be returned as multipart or event-stream responses",
                    );
                };

                if !self.schema.config.batching.enabled {
                    return self.bad_request_but_well_formed_graphql_over_http_request(
                        &request_context,
                        "batching is not enabled for this service",
                    );
                }

                if let Some(limit) = self.schema.config.batching.limit
                    && requests.len() > (limit as usize)
                {
                    return self.bad_request_but_well_formed_graphql_over_http_request(
                        &request_context,
                        format_args!("batch size exceeds limit of {limit}"),
                    );
                }

                self.runtime.metrics().record_batch_size(requests.len());

                let Some(responses) = self
                    .with_gateway_timeout(
                        futures_util::stream::iter(requests.into_iter())
                            .then(|request| self.execute_single(&request_context, request))
                            .collect::<Vec<_>>(),
                    )
                    .await
                else {
                    return self.gateway_timeout_error(&request_context);
                };

                Http::batch(format, responses)
            }
        }
    }

    pub(crate) fn execute_websocket_well_formed_graphql_request(
        self: &Arc<Self>,
        request_context: Arc<RequestContext>,
        request: Request,
    ) -> StreamResponse {
        self.execute_stream(request_context, request)
    }

    fn bad_request_but_well_formed_graphql_over_http_request(
        &self,
        request_context: &RequestContext,
        message: impl std::fmt::Display,
    ) -> http::Response<Body> {
        let error = GraphqlError::new(format!("Bad request: {message}"), ErrorCode::BadRequest);
        Http::error(
            request_context.response_format,
            Response::request_error(self.schema.config.error_code_mapping.clone(), [error])
                .with_extensions(default_response_extensions(&self.schema, request_context)),
        )
    }

    fn gateway_timeout_error(&self, request_context: &RequestContext) -> http::Response<Body> {
        Http::error(
            request_context.response_format,
            super::errors::response::gateway_timeout(self.schema.config.error_code_mapping.clone())
                .with_extensions(default_response_extensions(&self.schema, request_context)),
        )
    }
}
