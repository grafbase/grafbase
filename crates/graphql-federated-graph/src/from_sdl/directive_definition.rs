use super::{input_value_definition::convert_input_value_definition, *};

pub(super) fn ingest_directive_definition<'a>(
    directive_definition: ast::DirectiveDefinition<'a>,
    state: &mut State<'a>,
) -> Result<(), DomainError> {
    let (namespace, name) = split_namespace_name(directive_definition.name(), state);

    let mut locations = DirectiveLocations::default();

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

    let definition = DirectiveDefinitionRecord {
        namespace,
        name,
        locations,
        repeatable: directive_definition.is_repeatable(),
    };

    let directive_definition_id = state.graph.directive_definitions.len().into();
    state.graph.directive_definitions.push(definition);

    for argument in directive_definition.arguments() {
        let input_value_definition = convert_input_value_definition(argument, state)?;
        state
            .graph
            .push_directive_definition_argument(directive_definition_id, input_value_definition);
    }

    Ok(())
}
