mod create_one;
mod delete_many;
mod delete_one;
mod filter_many;
mod filter_one;

use super::{types, CreateTypeContext};
use crate::rules::visitor::VisitorContext;

pub(super) fn create(visitor_ctx: &mut VisitorContext<'_>, create_ctx: &CreateTypeContext<'_>) {
    let filter_input_type = types::filter::register_input(visitor_ctx, create_ctx);
    let filter_oneof_type = types::oneof::register_input(visitor_ctx, create_ctx);
    let create_input_type = types::create::register_input(visitor_ctx, create_ctx);
    let delete_output_type = types::delete::register_output(visitor_ctx, create_ctx);

    filter_one::create(visitor_ctx, create_ctx, &filter_oneof_type);
    filter_many::create(visitor_ctx, create_ctx, &filter_input_type);
    create_one::create(visitor_ctx, create_ctx, &create_input_type);
    delete_one::create(visitor_ctx, create_ctx, &filter_oneof_type, &delete_output_type);
    delete_many::create(visitor_ctx, create_ctx, &filter_input_type, &delete_output_type);
}
