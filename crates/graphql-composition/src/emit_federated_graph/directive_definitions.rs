use super::*;

pub(super) fn emit_directive_definitions(ir: &CompositionIr, ctx: &mut Context<'_>) {
    for definition in &ir.directive_definitions {
        let id = ctx.out.push_directive_definition(federated::DirectiveDefinitionRecord {
            namespace: None,
            name: definition.name,
            locations: definition.locations,
            repeatable: definition.repeatable,
        });

        for argument in &definition.arguments {
            let r#type = ctx.insert_field_type(ctx.subgraphs.walk(argument.r#type));
            let default = argument
                .default
                .as_ref()
                .map(|default| ctx.insert_value_with_type(default, r#type.definition.as_enum()));

            let argument = federated::InputValueDefinition {
                name: argument.name,
                r#type,
                directives: super::directive::transform_arbitray_type_directives(ctx, argument.directives.clone()),
                description: argument.description,
                default,
            };

            ctx.out.push_directive_definition_argument(id, argument);
        }
    }
}
