mod introspection;
mod operation_limits;

use crate::OperationContext;
use introspection::*;
use operation_limits::*;

use super::BindResult;

pub(super) fn validate(ctx: OperationContext<'_>) -> BindResult<()> {
    enforce_operation_limits(ctx)?;
    ensure_introspection_is_accepted(ctx)?;

    Ok(())
}
