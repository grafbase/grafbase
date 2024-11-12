mod builder;
mod error;
mod model;
mod query_modifications;

pub(crate) use error::*;
pub(crate) use model::*;
pub(crate) use query_modifications::*;

use crate::{
    prepare::{CachedOperation, PrepareContext},
    Runtime,
};

use super::Variables;

pub async fn plan(
    ctx: &mut PrepareContext<'_, impl Runtime>,
    operation: &CachedOperation,
    variables: &Variables,
) -> PlanResult<OperationPlan> {
    let query_modifications = QueryModifications::build(ctx, operation, variables).await?;
    OperationPlan::plan(ctx, operation, query_modifications)
}
