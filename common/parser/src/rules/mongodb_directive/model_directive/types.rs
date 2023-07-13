use super::create_type_context::CreateTypeContext;
use crate::rules::visitor::VisitorContext;

pub(super) fn register_oneof_type(visitor_ctx: &mut VisitorContext<'_>, create_ctx: &CreateTypeContext<'_>) -> String {
    crate::rules::model_directive::types::register_oneof_type(
        visitor_ctx,
        create_ctx.r#type,
        create_ctx.unique_directives(),
        Vec::new(),
    )
}
