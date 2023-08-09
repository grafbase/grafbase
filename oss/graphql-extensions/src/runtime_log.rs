use std::sync::Arc;

use dynaql::{
    extensions::{Extension, ExtensionContext, ExtensionFactory, NextExecute, NextPrepareRequest},
    parser::types::OperationDefinition,
    Positioned, Request, Response, ServerResult,
};
use grafbase_runtime::{
    log::{LogEventReceiver, LogEventType, OperationType},
    GraphqlRequestExecutionContext,
};

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
        let start = wasm_timer::SystemTime::now();

        let operation_name = request.operation_name.clone();
        let prepare_result = next.run(ctx, request).await;
        let end = wasm_timer::SystemTime::now();
        let duration: std::time::Duration = end.duration_since(start).unwrap();

        if prepare_result.is_err() {
            let request_id = &ctx
                .data::<GraphqlRequestExecutionContext>()
                .expect("must be set")
                .ray_id;

            self.log_event_receiver
                .invoke(
                    request_id,
                    LogEventType::BadRequest {
                        name: operation_name.as_deref(),
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
        use dynaql::parser::types::{OperationType as ParserOperationType, Selection};

        let request_id = &ctx
            .data::<GraphqlRequestExecutionContext>()
            .expect("must be set")
            .ray_id;

        self.log_event_receiver
            .invoke(request_id, LogEventType::OperationStarted { name: operation_name })
            .await;

        let start = wasm_timer::SystemTime::now();
        let response = next.run(ctx, operation_name, operation).await;
        let end = wasm_timer::SystemTime::now();
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
                request_id,
                LogEventType::OperationCompleted {
                    name: operation_name,
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
