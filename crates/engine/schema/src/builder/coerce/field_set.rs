use id_newtypes::IdRange;

use crate::{builder::GraphContext, DefinitionId, InputValueDefinitionId, InputValueSelection, InputValueSet};

use super::{value_path_to_string, ExtensionInputValueCoercer, ValuePathSegment};

#[derive(thiserror::Error, Debug)]
pub enum FieldSetError {
    #[error("Could not parse InputValueSet: {err}")]
    InvalidInputValueSet { err: String },
    #[error("Uknown input value '{name}'{path}")]
    UnknownInputValue { name: String, path: String },
    #[error("Cannot use fragments inside a InputValueSet")]
    CannotUseFragments,
    #[error("Type {ty} cannot have a selecction set{path}")]
    CannotHaveASelectionSet { ty: String, path: String },
    #[error("InputValueSet can only be used in directive applied on FIELD_DEFINITION | OBJECT | INTERFACE, but found on {location}")]
    InvalidInputValueSetOnLocation { location: &'static str },
}

impl ExtensionInputValueCoercer<'_, '_> {
    pub(crate) fn coerce_field_set(&mut self, selection_set: &str) -> Result<InputValueSet, FieldSetError> {
        let entity_id = match self.location {
            crate::builder::SchemaLocation::Object(id, _) => id.into(),
            crate::builder::SchemaLocation::Interface(id, _) => id.into(),
            crate::builder::SchemaLocation::FieldDefinition(id, _) => self.graph[id].parent_entity_id,
            _ => {
                return Err(FieldSetError::InvalidInputValueSetOnLocation {
                    location: self.location.to_cynic_location().as_str(),
                });
            }
        };
        let fields = format!("{{ {selection_set} }}");

        let doc = cynic_parser::parse_executable_document(&fields)
            .map_err(|err| FieldSetError::InvalidInputValueSet { err: err.to_string() })?;

        let selection_set = doc
            .operations()
            .next()
            .ok_or_else(|| FieldSetError::InvalidInputValueSet {
                err: "Could not find any seletion set".to_string(),
            })?
            .selection_set();

        let selection_set = convert_selection_set(
            self,
            self.graph[field_definition_id].argument_ids,
            selection_set,
            &mut Vec::new(),
        )?;
        Ok(InputValueSet::SelectionSet(selection_set))
    }
}

fn convert_selection_set(
    ctx: &GraphContext<'_>,
    possible_ids: IdRange<InputValueDefinitionId>,
    set: cynic_parser::executable::Iter<cynic_parser::executable::Selection>,
    value_path: &mut Vec<ValuePathSegment>,
) -> Result<Vec<InputValueSelection>, FieldSetError> {
    set.into_iter()
        .map(|selection| {
            let cynic_parser::executable::Selection::Field(field) = selection else {
                return Err(FieldSetError::CannotUseFragments);
            };
            let definition_id = possible_ids
                .into_iter()
                .find(|id| ctx.strings[ctx.graph[*id].name_id] == field.name())
                .ok_or_else(|| FieldSetError::UnknownInputValue {
                    name: field.name().to_string(),
                    path: value_path_to_string(ctx, value_path),
                })?;

            let subselection = if field.selection_set().len() == 0 {
                InputValueSet::All
            } else if let DefinitionId::InputObject(id) = ctx.graph[definition_id].ty_record.definition_id {
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
                return Err(FieldSetError::CannotHaveASelectionSet {
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
