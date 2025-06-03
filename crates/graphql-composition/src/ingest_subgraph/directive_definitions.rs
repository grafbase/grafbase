use super::*;
use crate::federated_graph::DirectiveLocations;

pub(super) fn ingest_directive_definition(
    ctx: &mut Context<'_>,
    directive_definition: ast::DirectiveDefinition<'_>,
    name: subgraphs::StringId,
) {
    let mut locations = DirectiveLocations::default();
    let subgraphs = &mut ctx.subgraphs;
    let subgraph_id = ctx.subgraph_id;

    for location in directive_definition.locations() {
        let location = match location {
            ast::DirectiveLocation::Query => DirectiveLocations::QUERY,
            ast::DirectiveLocation::Mutation => DirectiveLocations::MUTATION,
            ast::DirectiveLocation::Subscription => DirectiveLocations::SUBSCRIPTION,
            ast::DirectiveLocation::Field => DirectiveLocations::FIELD,
            ast::DirectiveLocation::FragmentDefinition => DirectiveLocations::FRAGMENT_DEFINITION,
            ast::DirectiveLocation::FragmentSpread => DirectiveLocations::FRAGMENT_SPREAD,
            ast::DirectiveLocation::InlineFragment => DirectiveLocations::INLINE_FRAGMENT,
            ast::DirectiveLocation::VariableDefinition => DirectiveLocations::VARIABLE_DEFINITION,
            ast::DirectiveLocation::Schema => DirectiveLocations::SCHEMA,
            ast::DirectiveLocation::Scalar => DirectiveLocations::SCALAR,
            ast::DirectiveLocation::Object => DirectiveLocations::OBJECT,
            ast::DirectiveLocation::FieldDefinition => DirectiveLocations::FIELD_DEFINITION,
            ast::DirectiveLocation::ArgumentDefinition => DirectiveLocations::ARGUMENT_DEFINITION,
            ast::DirectiveLocation::Interface => DirectiveLocations::INTERFACE,
            ast::DirectiveLocation::Union => DirectiveLocations::UNION,
            ast::DirectiveLocation::Enum => DirectiveLocations::ENUM,
            ast::DirectiveLocation::EnumValue => DirectiveLocations::ENUM_VALUE,
            ast::DirectiveLocation::InputObject => DirectiveLocations::INPUT_OBJECT,
            ast::DirectiveLocation::InputFieldDefinition => DirectiveLocations::INPUT_FIELD_DEFINITION,
        };

        locations |= location;
    }

    let mut arguments = Vec::with_capacity(directive_definition.arguments().len());

    for argument in directive_definition.arguments() {
        let argument_name = subgraphs.strings.intern(argument.name());
        let r#type = subgraphs.intern_field_type(argument.ty());
        let default_value = argument
            .default_value()
            .map(|default| crate::ast_value_to_subgraph_value(default, subgraphs));

        let directives = argument
            .directives()
            .map(|directive| subgraphs::InputValueDefinitionDirective {
                name: subgraphs.strings.intern(directive.name()),
                arguments: directive
                    .arguments()
                    .map(|arg| {
                        (
                            subgraphs.strings.intern(arg.name()),
                            crate::ast_value_to_subgraph_value(arg.value(), subgraphs),
                        )
                    })
                    .collect(),
            })
            .collect();

        arguments.push(subgraphs::InputValueDefinition {
            name: argument_name,
            r#type,
            default_value,
            directives,
        });
    }

    subgraphs.push_directive_definition(subgraphs::DirectiveDefinition {
        subgraph_id,
        name,
        locations,
        arguments,
        repeatable: directive_definition.is_repeatable(),
    });
}
