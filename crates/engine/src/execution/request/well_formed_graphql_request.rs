use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use error::{ErrorCode, GraphqlError};
use futures::StreamExt;
use hive_console_sdk::agent::usage_agent::{ExecutionReport, UsageAgentExt};
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
        let start = std::time::Instant::now();
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

                    if let (Some(hive_usage_reporter), Some(operation_attrs)) =
                        (&self.hive_usage_reporter, response.operation_attributes())
                    {
                        let error_count = response.error_code_counter().count();
                        let duration = start.elapsed();
                        if let Err(err) = hive_usage_reporter
                            .usage_agent
                            .add_report(ExecutionReport {
                                schema: hive_usage_reporter.schema.clone(),
                                client_name: request_context.client.as_ref().map(|c| c.name.clone()),
                                client_version: request_context.client.as_ref().and_then(|c| c.version.clone()),
                                timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64,
                                duration,
                                ok: error_count == 0,
                                errors: error_count,
                                operation_body: operation_attrs.sanitized_query.to_string(),
                                operation_name: operation_attrs.name.original().map(str::to_string),
                                persisted_document_hash: None,
                            })
                            .await
                        {
                            tracing::error!("Failed to send usage report to Hive: {err}");
                        }
                    }

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
