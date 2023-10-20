mod order_direction;
mod page_info;
mod scalar;
mod table;

use engine::registry::MetaEnumValue;

use super::context::{InputContext, OutputContext};

pub(super) fn generate(input_ctx: &InputContext<'_>, output_ctx: &mut OutputContext) {
    let tables = input_ctx
        .database_definition()
        .tables()
        .filter(|table| table.allowed_in_client());

    page_info::register(input_ctx, output_ctx);
    scalar::register(input_ctx, output_ctx);

    let direction_type = order_direction::register(input_ctx, output_ctx);

    for table in tables {
        table::generate(input_ctx, table, &direction_type, output_ctx);
    }

    for r#enum in input_ctx.database_definition().enums() {
        let type_name = input_ctx.type_name(r#enum.client_name());

        output_ctx.with_enum(&type_name, r#enum.id(), move |builder| {
            for variant in r#enum.variants() {
                let mut meta_value = MetaEnumValue::new(variant.client_name().to_string());
                meta_value.value = Some(variant.database_name().to_string());

                builder.push_variant(meta_value);
            }
        });
    }
}
