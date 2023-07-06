mod filter_one;

use super::{types, CreateTypeContext};
use crate::rules::visitor::VisitorContext;

pub(super) fn create(visitor_ctx: &mut VisitorContext<'_>, create_ctx: &CreateTypeContext<'_>) {
    let filter_oneof_type = types::register_oneof_type(visitor_ctx, create_ctx);
    filter_one::create(visitor_ctx, create_ctx, &filter_oneof_type);
}
