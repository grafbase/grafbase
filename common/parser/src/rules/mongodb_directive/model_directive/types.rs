use super::create_type_context::CreateTypeContext;
use crate::rules::visitor::VisitorContext;
use dynaql::registry::InputObjectType;

pub(super) fn register_oneof_type(visitor_ctx: &mut VisitorContext<'_>, create_ctx: &CreateTypeContext<'_>) -> String {
    let type_name = create_ctx.type_name();
    let input_type_name = format!("{type_name}ByInput");

    visitor_ctx.registry.get_mut().create_type(
        |registry| {
            let unique_fields = create_ctx
                .unique_directives()
                .map(|directive| directive.lookup_by_field(registry));

            let description = create_ctx.type_description().map(ToString::to_string);

            InputObjectType::new(input_type_name.clone(), unique_fields)
                .with_description(description)
                .with_oneof(true)
                .into()
        },
        &input_type_name,
        &input_type_name,
    );

    input_type_name
}
