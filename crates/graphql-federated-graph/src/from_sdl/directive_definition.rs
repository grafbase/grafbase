use super::*;

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

    let mut arguments_start: Option<InputValueDefinitionId> = None;
    let mut arguments_len = 0;

    for argument in directive_definition.arguments() {
        let id = ingest_input_value_definition(argument, state)?;

        if arguments_start.is_none() {
            arguments_start = Some(id);
        }

        arguments_len += 1;
    }

    let definition = DirectiveDefinition {
        namespace,
        name,
        locations,
        arguments: arguments_start
            .map(|arguments_start| (arguments_start, arguments_len))
            .unwrap_or(NO_INPUT_VALUE_DEFINITION),
        repeatable: directive_definition.is_repeatable(),
    };

    state.graph.directive_definitions.push(definition);

    Ok(())
}
