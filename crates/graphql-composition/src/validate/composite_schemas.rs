use super::*;

pub(super) mod source_schema;

pub(super) fn validate(ctx: &mut ValidateContext<'_>) {
    source_schema::query_root_type_inaccessible(ctx);
}
