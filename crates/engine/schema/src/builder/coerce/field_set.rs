use id_newtypes::IdRange;

use crate::{
    builder::GraphContext, CompositeTypeId, DefinitionId, EntityDefinitionId, FieldDefinitionId, FieldSetItemRecord, InputValueDefinitionId, InputValueSelection, InputValueSet, TypeRecord
};

use super::{value_path_to_string, ExtensionInputValueCoercer, InputValueError, ValuePathSegment};

#[derive(thiserror::Error, Debug)]
pub enum FieldSetError {
    #[error("Could not parse InputValueSet: {err}")]
    InvalidInputValueSet { err: String },
    #[error("Uknown field named '{name}' on type '{ty}'{path}")]
    UnknownField { name: String, ty: String, path: String },
    #[error("Uknown type named '{ty}'{path}")]
    UnknownType { ty: String, path: String },
    #[error("{ty} is not an object, interface or union{path}")]
    NotAnOutputType { ty: String, path: String },
    #[error("Type {ty} cannot have a selecction set{path}")]
    CannotHaveASelectionSet { ty: String, path: String },
    #[error("FieldSet can only be used in directive applied on FIELD_DEFINITION | OBJECT | INTERFACE | UNION, but found on {location}")]
    InvalidFieldSetOnLocation { location: &'static str },
    #[error("Invalid field argument{path}: {err}")]
    InvalidFieldArgument { err: InputValueError, path: String },
}

impl ExtensionInputValueCoercer<'_, '_> {
    pub(crate) fn coerce_field_set(&mut self, selection_set: &str) -> Result<InputValueSet, FieldSetError> {
        let composite_type_id: CompositeTypeId = match self.location {
            crate::builder::SchemaLocation::Object(id, _) => id.into(),
            crate::builder::SchemaLocation::Interface(id, _) => id.into(),
            crate::builder::SchemaLocation::FieldDefinition(id, _) => self.graph[id].parent_entity_id.into(),
            crate::builder::SchemaLocation::Union(id,_) => id.into(),
            _ => {
                return Err(FieldSetError::InvalidFieldSetOnLocation {
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

        let selection_set = convert_selection_set(self, composite_type_id, selection_set, &mut Vec::new())?;
        Ok(InputValueSet::SelectionSet(selection_set))
    }
}

fn convert_selection_set(
    ctx: &GraphContext<'_>,
    parent_field_output: CompositeTypeId,
    set: cynic_parser::executable::Iter<cynic_parser::executable::Selection>,
    value_path: &mut Vec<ValuePathSegment>,
) -> Result<Vec<InputValueSelection>, FieldSetError> {
    let mut out = Vec::new();
    let stack = vec![(parent_field_output, set)];
    while let Some((parent_entity_id, set)) = stack.pop() {
        for selection in set {
            match selection {
                cynic_parser::executable::Selection::Field(field) => {
        let field_definition_ids = match parent_entity_id {
            CompositeTypeId::Interface(id) => ctx.graph[id].field_ids,
            CompositeTypeId::Object(id) => ctx.graph[id].field_ids,
                        CompositeTypeId::Union(_) => todo!()
        };
                    let definition_id = field_definition_ids
                        .into_iter()
                        .find(|id| &ctx.strings[ctx.graph[*id].name_id] == field.name())
                        .ok_or_else(|| FieldSetError::UnknownField {
                            name: field.name().to_string(),
                            ty: ctx.type_name(TypeRecord {
                                definition_id: parent_entity_id.into(),
                                wrapping: Default::default(),
                            }),
                            path: value_path_to_string(ctx, value_path),
                        })?;
                    value_path.push(ctx.graph[definition_id].name_id.into());
                    out.push(convert_field(ctx, definition_id, field)?);
                    value_path.pop();
                }
                cynic_parser::executable::Selection::InlineFragment(fragment) => {
                    if let Some(type_condition) = fragment.type_condition() {
                        let definition_id = ctx
                            .graph
                            .type_definitions_ordered_by_name
                            .binary_search_by(|probe| {
                                match *probe {
                                    DefinitionId::Scalar(id) => &ctx.strings[ctx.graph[id].name_id],
                                    DefinitionId::Object(id) => &ctx.strings[ctx.graph[id].name_id],
                                    DefinitionId::Interface(id) => &ctx.strings[ctx.graph[id].name_id],
                                    DefinitionId::Union(id) => &ctx.strings[ctx.graph[id].name_id],
                                    DefinitionId::Enum(id) => &ctx.strings[ctx.graph[id].name_id],
                                    DefinitionId::InputObject(id) => &ctx.strings[ctx.graph[id].name_id],
                                }
                                .as_str()
                                .cmp(type_condition)
                            })
                            .map(|ix| ctx.graph.type_definitions_ordered_by_name[ix])
                            .map_err(|_| FieldSetError::UnknownType {
                                ty: type_condition.to_string(),
                                path: value_path_to_string(ctx, value_path),
                            })?;

                        let Some(entity_id) = definition_id.as_composite_type() else {
                            return Err(FieldSetError::NotAnOutputType {
                                ty: ctx.type_name(TypeRecord {
                                    definition_id,
                                    wrapping: Default::default(),
                                }),
                                path: value_path_to_string(ctx, value_path),
                            });
                        };

                        stack.push((entity_id, fragment.selection_set()));
                    } else {
                        stack.push((parent_field_output, fragment.selection_set()));
                    }
                }
                cynic_parser::executable::Selection::FragmentSpread(_) => todo!(),
            }
        }
    }
    // set.into_iter()
    //     .map(|selection| {
    //         let cynic_parser::executable::Selection::Field(field) = selection else {
    //             return Err(FieldSetError::CannotUseFragments);
    //         };
    //         let definition_id = field_ids
    //             .into_iter()
    //             .find(|id| ctx.strings[ctx.graph[*id].name_id] == field.name())
    //             .ok_or_else(|| FieldSetError::UnknownInputValue {
    //                 name: field.name().to_string(),
    //                 path: value_path_to_string(ctx, value_path),
    //             })?;
    //
    //         let subselection = if field.selection_set().len() == 0 {
    //             InputValueSet::All
    //         } else if let DefinitionId::InputObject(id) = ctx.graph[definition_id].ty_record.definition_id {
    //             value_path.push(ctx.graph[definition_id].name_id.into());
    //             let subselection = InputValueSet::SelectionSet(convert_selection_set(
    //                 ctx,
    //                 ctx.graph[id].input_field_ids,
    //                 field.selection_set(),
    //                 value_path,
    //             )?);
    //             value_path.pop();
    //             subselection
    //         } else {
    //             value_path.push(ctx.graph[definition_id].name_id.into());
    //             return Err(FieldSetError::CannotHaveASelectionSet {
    //                 ty: ctx.type_name(ctx.graph[definition_id].ty_record),
    //                 path: value_path_to_string(ctx, value_path),
    //             });
    //         };
    //
    //         Ok(InputValueSelection {
    //             definition_id,
    //             subselection,
    //         })
    //     })
    //     .collect()
    out
}

fn convert_field(
    ctx: &GraphContext<'_>,
    definition_id: FieldDefinitionId,
    field: cynic_parser::executable::FieldSelection<'_>,
) -> Result<FieldSetItemRecord, FieldSetError> {
    todo!()
}

pub fn has_non_empty_intersection_with(
    ctx: &GraphContext<'_>,
    left: EntityDefinitionId,
    right: EntityDefinitionId,
) -> bool {
    let left = match ;
    let right = other.possible_type_ids();
    let mut l = 0;
    let mut r = 0;
    while let (Some(left_id), Some(right_id)) = (left.get(l), right.get(r)) {
        match left_id.cmp(right_id) {
            std::cmp::Ordering::Less => l += 1,
            // At least one common object
            std::cmp::Ordering::Equal => return true,
            std::cmp::Ordering::Greater => r += 1,
        }
    }
    false
}
