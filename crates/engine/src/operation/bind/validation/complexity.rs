use schema::Schema;

use super::{BindResult, OperationWalker};

pub fn control_complexity(schema: &Schema, operation: OperationWalker<'_>) -> BindResult<()> {
    // TODO: If complexity control is completely turned off, just return.
    // Will deal with this when we have config for this...
}
