use crate::{BindError, BindResult, OperationContext};

pub(super) fn ensure_introspection_is_accepted(ctx: OperationContext<'_>) -> BindResult<()> {
    if ctx.operation.ty.is_query() && ctx.schema.settings.disable_introspection {
        for field in ctx.root_selection_set().fields() {
            if let Some(field) = field.as_data() {
                if ctx
                    .schema
                    .subgraphs
                    .introspection
                    .meta_fields
                    .contains(&field.definition_id)
                {
                    return Err(BindError::IntrospectionIsDisabled {
                        location: field.location,
                    });
                }
            }
        }
    }

    Ok(())
}
