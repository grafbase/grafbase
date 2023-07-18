use dynaql::{names::OUTPUT_FIELD_ID, registry::MetaInputValue};

use crate::rules::{
    mongodb_directive::model_directive::create_type_context::CreateTypeContext, visitor::VisitorContext,
};

pub(crate) fn register_input(visitor_ctx: &mut VisitorContext<'_>, create_ctx: &CreateTypeContext<'_>) -> String {
    let extra_fields = vec![MetaInputValue::new(OUTPUT_FIELD_ID, "ID").with_rename(Some("_id".to_string()))];

    crate::rules::model_directive::types::register_oneof_type(
        visitor_ctx,
        create_ctx.r#type,
        create_ctx.unique_directives(),
        extra_fields,
    )
}
