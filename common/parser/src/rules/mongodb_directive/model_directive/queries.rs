mod create_one;
mod delete_one;
mod filter_one;

use super::{types, CreateTypeContext};
use crate::rules::visitor::VisitorContext;

pub(super) fn create(visitor_ctx: &mut VisitorContext<'_>, create_ctx: &CreateTypeContext<'_>) {
    let filter_oneof_type = types::register_oneof_type(visitor_ctx, create_ctx);
    let create_input_type = types::register_create_input_type(visitor_ctx, create_ctx);

    filter_one::create(visitor_ctx, create_ctx, &filter_oneof_type);
    create_one::create(visitor_ctx, create_ctx, &create_input_type);
    delete_one::create(visitor_ctx, create_ctx, &filter_oneof_type);
}
