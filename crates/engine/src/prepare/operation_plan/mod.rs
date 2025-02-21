mod builder;
mod error;
mod model;
mod query_modifications;

pub(crate) use error::*;
pub(crate) use model::*;
use operation::Variables;
pub(crate) use query_modifications::*;

use crate::{
    Runtime,
    prepare::{CachedOperation, PrepareContext},
};

#[tracing::instrument(name = "plan", level = "debug", skip_all)]
pub async fn plan(
    ctx: &mut PrepareContext<'_, impl Runtime>,
    operation: &CachedOperation,
    variables: &Variables,
) -> PlanResult<OperationPlan> {
    let query_modifications = QueryModifications::build(ctx, operation, variables).await?;
    OperationPlan::plan(ctx, operation, query_modifications)
}
