mod context;
mod error;
mod operation;
mod trusted_documents;

use std::sync::Arc;

pub(crate) use context::*;

use grafbase_telemetry::graphql::{GraphqlOperationAttributes, OperationType};
use runtime::hooks::Hooks;
use tracing::{info_span, Instrument};

use crate::{
    operation::{OperationPlan, SolvedOperation, Variables},
    request::Request,
    response::Response,
    Runtime,
};

impl<'ctx, R: Runtime> PrepareContext<'ctx, R> {
    pub(crate) async fn prepare_operation(
        &mut self,
        request: Request,
    ) -> Result<PreparedOperation, Response<<R::Hooks as Hooks>::OnOperationResponseOutput>> {
        let span = info_span!("prepare operation");
        let result = self.prepare_operation_inner(request).instrument(span).await;
        let duration = self.executed_operation_builder.track_prepare();

        match result {
            Ok(operation) => {
                self.metrics()
                    .record_successful_preparation_duration(operation.cached.attributes.clone(), duration);

                Ok(operation)
            }
            Err(response) => {
                self.metrics()
                    .record_failed_preparation_duration(response.operation_attributes().cloned(), duration);

                Err(response)
            }
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct CachedOperation {
    pub solved: SolvedOperation,
    pub attributes: GraphqlOperationAttributes,
}

impl CachedOperation {
    pub(crate) fn ty(&self) -> OperationType {
        self.attributes.ty
    }
}

pub(crate) struct PreparedOperation {
    pub cached: Arc<CachedOperation>,
    pub plan: OperationPlan,
    pub variables: Variables,
}
