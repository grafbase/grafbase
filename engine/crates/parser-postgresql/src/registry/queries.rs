mod find_one;
mod input;

use super::context::{InputContext, OutputContext};

pub(super) fn generate(input_ctx: &InputContext<'_>, output_ctx: &mut OutputContext) {
    let tables = input_ctx
        .database_definition()
        .tables()
        .filter(|table| table.allowed_in_client());

    for table in tables {
        let filter_oneof_type = input::register_oneof_filter(input_ctx, table, output_ctx);

        find_one::register(input_ctx, table, &filter_oneof_type, output_ctx);
    }
}
