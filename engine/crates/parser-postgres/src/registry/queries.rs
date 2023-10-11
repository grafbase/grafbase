mod create_many;
mod create_one;
mod delete_many;
mod delete_one;
mod find_many;
mod find_one;
mod input;
mod update_many;
mod update_one;

use super::context::{InputContext, OutputContext};

pub(super) fn generate(input_ctx: &InputContext<'_>, output_ctx: &mut OutputContext) {
    let tables = input_ctx
        .database_definition()
        .tables()
        .filter(|table| table.allowed_in_client());

    for table in tables {
        let filter_oneof_type = input::oneof::register(input_ctx, table, output_ctx);
        let create_input_type = input::create::register(input_ctx, table, output_ctx);
        let update_input_type = input::update::register(input_ctx, table, output_ctx);

        let (simple_filter, complex_filter) = input::filter::register(input_ctx, table, output_ctx);

        find_one::register(input_ctx, table, &filter_oneof_type, output_ctx);
        find_many::register(input_ctx, table, &complex_filter, output_ctx);
        delete_one::register(input_ctx, table, &filter_oneof_type, output_ctx);
        delete_many::register(input_ctx, table, &simple_filter, output_ctx);
        create_one::register(input_ctx, table, &create_input_type, output_ctx);
        create_many::register(input_ctx, table, &create_input_type, output_ctx);
        update_one::register(input_ctx, table, &filter_oneof_type, &update_input_type, output_ctx);
        update_many::register(input_ctx, table, &simple_filter, &update_input_type, output_ctx);
    }
}
