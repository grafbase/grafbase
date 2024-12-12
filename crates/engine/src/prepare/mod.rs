mod context;
mod error;
mod operation;
mod trusted_documents;

use std::sync::Arc;

pub(crate) use context::*;

use grafbase_telemetry::graphql::{GraphqlOperationAttributes, OperationName, OperationType};
use runtime::hooks::Hooks;
use tracing::{info_span, Instrument};

use crate::{
    engine::cache::DocumentKey,
    operation::{BoundOperation, OperationPlan, SolvedOperation, Variables},
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
                    .record_successful_preparation_duration(operation.attributes(), duration);

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
pub struct CachedOperation {
    pub(crate) solved: SolvedOperation,
    pub(crate) attributes: CachedOperationAttributes,
    // This is optional because we only currently need it for complexity control
    // That may change in the future...
    pub(crate) operation: Option<BoundOperation>,

    pub(crate) document_key: DocumentKey<'static>,
    pub(crate) document: String,
    pub(crate) operation_name: Option<String>,
}

impl CachedOperation {
    pub(crate) fn ty(&self) -> OperationType {
        self.attributes.ty
    }

    /// Should be used when a request has errored and we only have the cached attributes
    pub(crate) fn operation_attributes_for_error(&self) -> GraphqlOperationAttributes {
        self.attributes.clone().attributes_for_error()
    }
}

/// The set of Operation attributes that can be cached
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct CachedOperationAttributes {
    pub ty: OperationType,
    pub name: OperationName,
    pub sanitized_query: Arc<str>,
}

impl CachedOperationAttributes {
    pub fn attributes_for_error(self) -> GraphqlOperationAttributes {
        let CachedOperationAttributes {
            ty,
            name,
            sanitized_query,
        } = self;

        GraphqlOperationAttributes {
            ty,
            name,
            sanitized_query,
            complexity: None,
        }
    }
}

pub(crate) struct PreparedOperation {
    pub cached: Arc<CachedOperation>,
    pub plan: OperationPlan,
    pub variables: Variables,
    pub complexity: Option<usize>,
}

impl PreparedOperation {
    pub fn attributes(&self) -> GraphqlOperationAttributes {
        let CachedOperationAttributes {
            ty,
            name,
            sanitized_query,
        } = self.cached.attributes.clone();

        GraphqlOperationAttributes {
            ty,
            name,
            sanitized_query,
            complexity: self.complexity,
        }
    }
}
