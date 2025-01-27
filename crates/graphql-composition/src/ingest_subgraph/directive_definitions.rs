use super::*;

pub(super) fn ingest_directive_definition(
    directive_definition: ast::DirectiveDefinition<'_>,
    subgraph_id: SubgraphId,
    subgraphs: &mut Subgraphs,
) {
    let name = subgraphs.strings.intern(directive_definition.name());
    let mut locations = subgraphs::DirectiveLocations::default();

    for location in directive_definition.locations() {
        let location = match location {
            ast::DirectiveLocation::Query => subgraphs::DirectiveLocations::QUERY,
            ast::DirectiveLocation::Mutation => subgraphs::DirectiveLocations::MUTATION,
            ast::DirectiveLocation::Subscription => subgraphs::DirectiveLocations::SUBSCRIPTION,
            ast::DirectiveLocation::Field => subgraphs::DirectiveLocations::FIELD,
            ast::DirectiveLocation::FragmentDefinition => subgraphs::DirectiveLocations::FRAGMENT_DEFINITION,
            ast::DirectiveLocation::FragmentSpread => subgraphs::DirectiveLocations::FRAGMENT_SPREAD,
            ast::DirectiveLocation::InlineFragment => subgraphs::DirectiveLocations::INLINE_FRAGMENT,
            ast::DirectiveLocation::VariableDefinition => subgraphs::DirectiveLocations::VARIABLE_DEFINITION,
            ast::DirectiveLocation::Schema => subgraphs::DirectiveLocations::SCHEMA,
            ast::DirectiveLocation::Scalar => subgraphs::DirectiveLocations::SCALAR,
            ast::DirectiveLocation::Object => subgraphs::DirectiveLocations::OBJECT,
            ast::DirectiveLocation::FieldDefinition => subgraphs::DirectiveLocations::FIELD_DEFINITION,
            ast::DirectiveLocation::ArgumentDefinition => subgraphs::DirectiveLocations::ARGUMENT_DEFINITION,
            ast::DirectiveLocation::Interface => subgraphs::DirectiveLocations::INTERFACE,
            ast::DirectiveLocation::Union => subgraphs::DirectiveLocations::UNION,
            ast::DirectiveLocation::Enum => subgraphs::DirectiveLocations::ENUM,
            ast::DirectiveLocation::EnumValue => subgraphs::DirectiveLocations::ENUM_VALUE,
            ast::DirectiveLocation::InputObject => subgraphs::DirectiveLocations::INPUT_OBJECT,
            ast::DirectiveLocation::InputFieldDefinition => subgraphs::DirectiveLocations::INPUT_FIELD_DEFINITION,
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
            .map(|directive| subgraphs::Directive {
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
    });
}
