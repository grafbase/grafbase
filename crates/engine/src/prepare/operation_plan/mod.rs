mod builder;
mod error;
mod model;
mod query_modifications;

use ::error::ErrorResponse;
pub(crate) use error::*;
pub(crate) use model::*;
use operation::Variables;
pub(crate) use query_modifications::*;

use crate::{
    ErrorCode, Runtime,
    prepare::{CachedOperation, PrepareContext},
    response::{GraphqlError, Response},
};

#[tracing::instrument(name = "plan", level = "debug", skip_all)]
pub async fn plan<OnOperationResponseHookOutput>(
    ctx: &mut PrepareContext<'_, impl Runtime>,
    operation: &CachedOperation,
    variables: &Variables,
) -> Result<OperationPlan, Response<OnOperationResponseHookOutput>> {
    async move {
        let query_modifications = QueryModifications::build(ctx, operation, variables).await?;
        OperationPlan::plan(ctx, operation, query_modifications).await
    }
    .await
    .map_err(|error| match error {
        PlanError::Internal => Response::request_error([GraphqlError::new(
            "Could not plan operation",
            ErrorCode::OperationPlanningError,
        )]),
        PlanError::GraphqlError(error) => Response::request_error([error]),
        PlanError::ErrorResponse(ErrorResponse { status, errors }) => Response::refuse_request_with(status, errors),
    })
}
