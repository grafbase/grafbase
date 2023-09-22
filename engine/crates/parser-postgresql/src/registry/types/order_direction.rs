use engine::registry::{EnumType, MetaEnumValue};

use crate::registry::context::{InputContext, OutputContext};

pub(super) fn register(input_ctx: &InputContext<'_>, output_ctx: &mut OutputContext) -> String {
    let type_name = input_ctx.type_name("OrderByDirection");

    let variants = ["ASC", "DESC"].iter().map(|name| {
        let mut variant = MetaEnumValue::new((*name).to_string());
        variant.value = Some((*name).to_string());

        variant
    });

    let r#enum = EnumType::new(type_name.to_string(), variants);

    output_ctx.create_enum_type(r#enum);
    type_name.into_owned()
}
