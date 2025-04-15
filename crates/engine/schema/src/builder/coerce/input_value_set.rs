use id_newtypes::IdRange;

use crate::{
    InputValueDefinitionId, InputValueSelection, InputValueSet, TypeDefinitionId,
    builder::{GraphBuilder, sdl},
};

use super::{ExtensionDirectiveArgumentsCoercer, ValuePathSegment, value_path_to_string};

#[derive(thiserror::Error, Debug)]
pub enum InputValueSetError {
    #[error("Could not parse InputValueSet: {err}")]
    InvalidInputValueSet { err: String },
    #[error("Unknown input value '{name}'{path}")]
    UnknownInputValue { name: String, path: String },
    #[error("Cannot use fragments inside a InputValueSet")]
    CannotUseFragments,
    #[error("Type {ty} cannot have a selecction set{path}")]
    CannotHaveASelectionSet { ty: String, path: String },
    #[error("InputValueSet can only be used in directive applied on FIELD_DEFINITION, but found on {location}")]
    InvalidInputValueSetOnLocation { location: &'static str },
}

impl ExtensionDirectiveArgumentsCoercer<'_, '_> {
    pub(crate) fn coerce_input_value_set(&mut self, selection_set: &str) -> Result<InputValueSet, InputValueSetError> {
        let sdl::SdlDefinition::FieldDefinition(field_definition) = self.current_definition else {
            return Err(InputValueSetError::InvalidInputValueSetOnLocation {
                location: self.current_definition.location().as_str(),
            });
        };
        if selection_set.trim() == "*" {
            return Ok(InputValueSet::All);
        }
        let fields = format!("{{ {selection_set} }}");

        let doc = cynic_parser::parse_executable_document(&fields)
            .map_err(|err| InputValueSetError::InvalidInputValueSet { err: err.to_string() })?;

        let selection_set = doc
            .operations()
            .next()
            .ok_or_else(|| InputValueSetError::InvalidInputValueSet {
                err: "Could not find any seletion set".to_string(),
            })?
            .selection_set();

        let selection_set = convert_selection_set(
            self,
            self.graph[field_definition.id].argument_ids,
            selection_set,
            &mut Vec::new(),
        )?;
        Ok(InputValueSet::SelectionSet(selection_set))
    }
}

fn convert_selection_set(
    ctx: &GraphBuilder<'_>,
    possible_ids: IdRange<InputValueDefinitionId>,
    set: cynic_parser::executable::Iter<cynic_parser::executable::Selection>,
    value_path: &mut Vec<ValuePathSegment>,
) -> Result<Vec<InputValueSelection>, InputValueSetError> {
    set.into_iter()
        .map(|selection| {
            let cynic_parser::executable::Selection::Field(field) = selection else {
                return Err(InputValueSetError::CannotUseFragments);
            };
            let definition_id = possible_ids
                .into_iter()
                .find(|id| ctx[ctx.graph[*id].name_id] == field.name())
                .ok_or_else(|| InputValueSetError::UnknownInputValue {
                    name: field.name().to_string(),
                    path: value_path_to_string(ctx, value_path),
                })?;

            let subselection = if field.selection_set().len() == 0 {
                InputValueSet::All
            } else if let TypeDefinitionId::InputObject(id) = ctx.graph[definition_id].ty_record.definition_id {
                value_path.push(ctx.graph[definition_id].name_id.into());
                let subselection = InputValueSet::SelectionSet(convert_selection_set(
                    ctx,
                    ctx.graph[id].input_field_ids,
                    field.selection_set(),
                    value_path,
                )?);
                value_path.pop();
                subselection
            } else {
                value_path.push(ctx.graph[definition_id].name_id.into());
                return Err(InputValueSetError::CannotHaveASelectionSet {
                    ty: ctx.type_name(ctx.graph[definition_id].ty_record),
                    path: value_path_to_string(ctx, value_path),
                });
            };

            Ok(InputValueSelection {
                definition_id,
                subselection,
            })
        })
        .collect()
}
