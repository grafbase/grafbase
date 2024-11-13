mod complexity;
mod introspection;
mod operation_limits;

use crate::operation::OperationWalker;
use complexity::control_complexity;
use introspection::*;
use operation_limits::*;
use schema::Schema;

use super::BindResult;

pub(super) fn validate(schema: &Schema, operation: OperationWalker<'_>) -> BindResult<()> {
    enforce_operation_limits(schema, operation)?;
    ensure_introspection_is_accepted(schema, operation)?;
    control_complexity(schema, operation)?;

    Ok(())
}
