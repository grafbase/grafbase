use dynaql::registry::{InputObjectType, MetaInputValue};
use dynaql_parser::types::TypeDefinition;

use crate::{
    registry::names::MetaNames,
    rules::{unique_directive::UniqueDirective, visitor::VisitorContext},
};

pub(crate) fn register_oneof_type<'a>(
    visitor_ctx: &'a mut VisitorContext<'_>,
    r#type: &'a TypeDefinition,
    unique_directives: impl IntoIterator<Item = &'a UniqueDirective> + 'a,
    extra_fields: impl IntoIterator<Item = MetaInputValue> + 'a,
) -> String {
    let input_type_name = MetaNames::by_input(r#type);

    visitor_ctx.registry.get_mut().create_type(
        |registry| {
            let unique_fields = unique_directives
                .into_iter()
                .map(|directive| directive.lookup_by_field(registry));

            let description = r#type.description().map(ToString::to_string);
            let fields = extra_fields.into_iter().chain(unique_fields);

            InputObjectType::new(input_type_name.clone(), fields)
                .with_description(description)
                .with_oneof(true)
                .into()
        },
        &input_type_name,
        &input_type_name,
    );

    input_type_name
}
