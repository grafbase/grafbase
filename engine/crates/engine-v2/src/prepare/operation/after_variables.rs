use std::sync::Arc;

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
        let plan = match crate::plan::plan_solution(self, &cached_operation, &variables).await {
            Ok(plan) => plan,
            Err(err) => {
                return Err(PrepareError::Plan {
                    attributes: Box::new(Some(cached_operation.attributes.clone())),
                    err,
                })
            }
        };

        Ok(PreparedOperation {
            cached: cached_operation,
            plan,
            variables,
        })
    }
}
