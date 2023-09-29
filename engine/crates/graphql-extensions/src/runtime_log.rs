use std::sync::Arc;

use common_types::{LogEventType, OperationType};
use engine::{
    extensions::{Extension, ExtensionContext, ExtensionFactory, NextExecute, NextPrepareRequest},
    parser::types::OperationDefinition,
    Positioned, Request, Response, ServerResult,
};
use runtime::log::LogEventReceiver;

pub struct RuntimeLogExtension {
    log_event_receiver: Arc<Box<dyn LogEventReceiver + Send + Sync>>,
}

impl RuntimeLogExtension {
    pub fn new(receiver: Box<dyn LogEventReceiver + Send + Sync>) -> Self {
        Self {
            log_event_receiver: Arc::new(receiver),
        }
    }
}

impl ExtensionFactory for RuntimeLogExtension {
    fn create(&self) -> Arc<dyn Extension> {
        Arc::new(Self {
            log_event_receiver: self.log_event_receiver.clone(),
        })
    }
}

#[async_trait::async_trait]
impl Extension for RuntimeLogExtension {
    /// Called at prepare request.
    async fn prepare_request(
        &self,
        ctx: &ExtensionContext<'_>,
        request: Request,
        next: NextPrepareRequest<'_>,
    ) -> ServerResult<Request> {
        let start = web_time::SystemTime::now();

        let operation_name = request.operation_name.clone();
        let prepare_result = next.run(ctx, request).await;
        let end = web_time::SystemTime::now();
        let duration: std::time::Duration = end.duration_since(start).unwrap();

        if prepare_result.is_err() {
            let runtime_ctx = ctx.data::<runtime::Context>().expect("must be set");

            self.log_event_receiver
                .invoke(
                    runtime_ctx.ray_id(),
                    runtime_ctx.log.request_log_event_id,
                    LogEventType::BadRequest {
                        name: operation_name.as_deref().map(From::from),
                        duration,
                    },
                )
                .await;
        }

        prepare_result
    }

    /// Called at execute query.
    async fn execute(
        &self,
        ctx: &ExtensionContext<'_>,
        operation_name: Option<&str>,
        operation: &OperationDefinition,
        next: NextExecute<'_>,
    ) -> Response {
        use engine::parser::types::{OperationType as ParserOperationType, Selection};

        let runtime_ctx = ctx.data::<runtime::Context>().expect("must be set");

        self.log_event_receiver
            .invoke(
                runtime_ctx.ray_id(),
                runtime_ctx.log.request_log_event_id,
                LogEventType::OperationStarted {
                    name: operation_name.map(From::from),
                },
            )
            .await;

        let start = web_time::SystemTime::now();
        let response = next.run(ctx, operation_name, operation).await;
        let end = web_time::SystemTime::now();
        let duration: std::time::Duration = end.duration_since(start).unwrap();

        let operation_name = operation_name.or_else(|| match operation.selection_set.node.items.as_slice() {
            [Positioned {
                node: Selection::Field(field),
                ..
            }] => Some(field.node.name.node.as_str()),
            _ => None,
        });

        self.log_event_receiver
            .invoke(
                runtime_ctx.ray_id(),
                runtime_ctx.log.request_log_event_id,
                LogEventType::OperationCompleted {
                    name: operation_name.map(From::from),
                    duration,
                    r#type: match response.operation_type {
                        ParserOperationType::Query => OperationType::Query {
                            is_introspection: crate::is_operation_introspection(operation),
                        },
                        ParserOperationType::Mutation => OperationType::Mutation,
                        ParserOperationType::Subscription => OperationType::Subscription,
                    },
                },
            )
            .await;

        response
    }
}
