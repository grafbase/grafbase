use graphql_federated_graph::NO_INPUT_VALUE_DEFINITION;

use super::*;

pub(super) fn compose_directive_definitions(ctx: &mut Context<'_>) {
    // Filtered definitions. Sort by name, dedup.
    let mut definitions: Vec<&subgraphs::DirectiveDefinition> = ctx
        .subgraphs
        .directive_definitions()
        .iter()
        .filter(|definition| ctx.subgraphs.is_composed_directive(definition.name))
        .collect();

    definitions.sort_by_key(|definition| definition.name);
    definitions.dedup_by_key(|definition| definition.name);

    // Emit
    for definition in definitions {
        let name = ctx.insert_string(definition.name);
        let mut arguments_start: Option<federated::InputValueDefinitionId> = None;
        let mut arguments_len = 0;

        for argument in &definition.arguments {
            let input_value_definition = ir::InputValueDefinitionIr {
                name: ctx.insert_string(argument.name),
                r#type: argument.r#type,
                directives: argument
                    .directives
                    .iter()
                    .map(|directive| ir::Directive::Other {
                        name: ctx.insert_string(directive.name),
                        arguments: directive
                            .arguments
                            .iter()
                            .map(|(name, value)| (ctx.insert_string(*name), value.clone()))
                            .collect(),
                    })
                    .collect(),
                description: None,
                default: argument.default_value.clone(),
            };

            let id = ctx.insert_input_value_definition(input_value_definition);
            arguments_len += 1;

            if arguments_start.is_none() {
                arguments_start = Some(id);
            }
        }

        let arguments = arguments_start
            .map(|start| -> federated::InputValueDefinitions { (start, arguments_len) })
            .unwrap_or(NO_INPUT_VALUE_DEFINITION);

        ctx.insert_directive_definition(ir::DirectiveDefinitionIr {
            name,
            locations: definition.locations,
            arguments,
            repeatable: definition.repeatable,
        });
    }
}
