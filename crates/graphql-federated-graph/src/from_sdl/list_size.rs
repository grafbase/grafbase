use cynic_parser_deser::ConstDeserializer;

use crate::{directives::ListSizeDirective, InputValueDefinitionId, ListSize};

use super::{ast, Definition, DomainError, State};

pub fn ingest_list_size_directive<'a>(
    parent_id: Definition,
    fields: impl Iterator<Item = ast::FieldDefinition<'a>>,
    state: &mut State<'a>,
) -> Result<(), DomainError> {
    for field in fields {
        let directive = field
            .directives()
            .find(|field| field.name() == "listSize")
            .and_then(|directive| directive.deserialize::<ListSizeDirective>().ok());

        let Some(directive) = directive else { continue };
        let Some(field_id) = state.selection_map.get(&(parent_id, field.name())).copied() else {
            continue;
        };
        let field = &state.fields[usize::from(field_id)];

        let ListSizeDirective {
            assumed_size,
            slicing_arguments,
            sized_fields,
            require_one_slicing_argument,
        } = directive;

        let argument_base_index = usize::from(field.arguments.0);
        let arguments = &state.input_value_definitions[argument_base_index..argument_base_index + field.arguments.1];
        let slicing_arguments = slicing_arguments
            .iter()
            .filter_map(|argument| {
                let (index, _) = arguments
                    .iter()
                    .enumerate()
                    .find(|(_, value)| state[value.name] == *argument)?;

                Some(InputValueDefinitionId::from(index + argument_base_index))
            })
            .collect();

        let child_type_id = field.r#type.definition;
        let sized_fields = sized_fields
            .iter()
            .filter_map(|field| state.selection_map.get(&(child_type_id, field)).copied())
            .collect();

        state.list_sizes.push((
            field_id,
            ListSize {
                assumed_size,
                slicing_arguments,
                sized_fields,
                require_one_slicing_argument,
            },
        ));
    }

    Ok(())
}
