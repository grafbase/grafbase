use std::sync::Arc;

use super::complexity_control;
use crate::{
    operation::Variables,
    prepare::{
        error::{PrepareError, PrepareResult},
        CachedOperation, PrepareContext, PreparedOperation,
    },
    Runtime,
};

impl<'ctx, R: Runtime> PrepareContext<'ctx, R> {
    pub(super) async fn prepare_cached_operation(
        &mut self,
        cached_operation: Arc<CachedOperation>,
        variables: Variables,
    ) -> PrepareResult<PreparedOperation> {
        let plan = match crate::operation::plan(self, &cached_operation, &variables).await {
            Ok(plan) => plan,
            Err(err) => {
                return Err(PrepareError::Plan {
                    attributes: Box::new(Some(cached_operation.attributes.clone())),
                    err,
                })
            }
        };

        if !self.schema().settings.complexity_control.is_disabled() {
            let operation = cached_operation
                .operation
                .as_ref()
                .expect("cached_operation to be present if complexity control is active");

            complexity_control::control_complexity(self.schema(), operation.walker_with(self.schema()), &variables)?;
        }

        Ok(PreparedOperation {
            cached: cached_operation,
            plan,
            variables,
        })
    }
}
