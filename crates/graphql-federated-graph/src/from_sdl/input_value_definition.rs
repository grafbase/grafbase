use super::*;

pub(super) fn convert_input_value_definition<'a>(
    input_value_definition: ast::InputValueDefinition<'a>,
    state: &mut State<'a>,
) -> Result<InputValueDefinition, DomainError> {
    let name = state.insert_string(input_value_definition.name());
    let r#type = state.field_type(input_value_definition.ty())?;
    let directives = collect_input_value_directives(input_value_definition.directives(), state)?;
    let description = input_value_definition
        .description()
        .map(|description| state.insert_string(&description.to_cow()));
    let default = input_value_definition
        .default_value()
        .map(|default| state.insert_value(default, r#type.definition.as_enum()));

    Ok(InputValueDefinition {
        name,
        r#type,
        directives,
        description,
        default,
    })
}

pub(super) fn ingest_input_value_definition<'a>(
    input_value_definition: ast::InputValueDefinition<'a>,
    state: &mut State<'a>,
) -> Result<InputValueDefinitionId, DomainError> {
    let input_value_definition = convert_input_value_definition(input_value_definition, state)?;
    let id = state.graph.push_input_value_definition(input_value_definition);

    Ok(id)
}
