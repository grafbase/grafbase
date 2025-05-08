use id_newtypes::IdRange;

use crate::{
    CompositeTypeId, FieldDefinitionId, FieldSetItemRecord, FieldSetRecord, InputValueDefinitionRecord,
    ObjectDefinitionId, SchemaFieldArgumentId, SchemaFieldArgumentRecord, SchemaFieldRecord, TypeDefinitionId,
    TypeRecord,
    builder::{GraphBuilder, sdl},
};

use super::{ExtensionDirectiveArgumentsCoercer, InputValueError, ValuePathSegment, value_path_to_string};

#[derive(thiserror::Error, Debug)]
pub(crate) enum FieldSetError {
    #[error("Failed to coerce argument{path}: {err}")]
    InputValueError { err: InputValueError, path: String },
    #[error("Could not parse InputValueSet: {err}")]
    InvalidFieldSet { err: String },
    #[error("Unknown field '{name}' on type '{ty}'{path}")]
    UnknownField { name: String, ty: String, path: String },
    #[error("Unknown type '{ty}'{path}")]
    UnknownType { ty: String, path: String },
    #[error("{ty} is not an object, interface or union{path}")]
    NotAnOutputType { ty: String, path: String },
    #[error(
        "FieldSet can only be used in directive applied on FIELD_DEFINITION | OBJECT | INTERFACE | UNION, but found on {location}"
    )]
    InvalidFieldSetOnLocation { location: &'static str },
    #[error(
        "Field '{name}'{path} does not exists on {ty}, it's a union. Only interfaces and objects have fields, consider using a fragment with a type condition."
    )]
    UnionHaveNoFields { name: String, ty: String, path: String },
    #[error("Type condition on '{name}' cannot be used in a '{parent}' selection_set{path}")]
    DisjointTypeCondition { parent: String, name: String, path: String },
    #[error("Cannot use named fragments inside a FieldSet")]
    CannotUseNamedFragments,
    #[error("Field '{name}'{path} cannot have a selection set, it's a {ty}. Only interfaces, unions and objects can.")]
    CannotHaveSelectionSet { name: String, ty: String, path: String },
    #[error("Leaf field '{name}'{path} must be a scalar or an enum, but is a {ty}.")]
    LeafMustBeAScalarOrEnum { name: String, ty: String, path: String },
}

impl ExtensionDirectiveArgumentsCoercer<'_, '_> {
    pub(crate) fn coerce_field_set(&mut self, selection_set: &str) -> Result<FieldSetRecord, FieldSetError> {
        let composite_type_id: CompositeTypeId = match self.current_definition {
            sdl::SdlDefinition::Object(def) => def.id.into(),
            sdl::SdlDefinition::Interface(def) => def.id.into(),
            sdl::SdlDefinition::FieldDefinition(def) => self.graph[def.id].parent_entity_id.into(),
            sdl::SdlDefinition::Union(def) => def.id.into(),
            _ => {
                return if self.is_default_value {
                    Ok(FieldSetRecord::default())
                } else {
                    Err(FieldSetError::InvalidFieldSetOnLocation {
                        location: self.current_definition.location().as_str(),
                    })
                };
            }
        };
        self.ctx.parse_field_set(composite_type_id, selection_set)
    }
}

impl GraphBuilder<'_> {
    pub(crate) fn parse_field_set(
        &mut self,
        parent: CompositeTypeId,
        selection_set: &str,
    ) -> Result<FieldSetRecord, FieldSetError> {
        let fields = format!("{{ {selection_set} }}");
        let doc = cynic_parser::parse_executable_document(&fields)
            .map_err(|err| FieldSetError::InvalidFieldSet { err: err.to_string() })?;
        let selection_set = doc
            .operations()
            .next()
            .ok_or_else(|| FieldSetError::InvalidFieldSet {
                err: "Could not find any seletion set".to_string(),
            })?
            .selection_set();
        convert_selection_set(self, parent, selection_set, &mut Vec::new())
    }
}

fn convert_selection_set<'a>(
    ctx: &mut GraphBuilder<'_>,
    parent_field_output: CompositeTypeId,
    set: cynic_parser::executable::Iter<'a, cynic_parser::executable::Selection<'a>>,
    value_path: &mut Vec<ValuePathSegment>,
) -> Result<FieldSetRecord, FieldSetError> {
    let mut out = Vec::new();
    let mut stack = vec![(parent_field_output, set)];
    while let Some((parent_composite_type_id, set)) = stack.pop() {
        for selection in set {
            match selection {
                cynic_parser::executable::Selection::Field(field) => {
                    let field_definition_ids = match parent_composite_type_id {
                        CompositeTypeId::Interface(id) => ctx.graph[id].field_ids,
                        CompositeTypeId::Object(id) => ctx.graph[id].field_ids,
                        CompositeTypeId::Union(id) => {
                            return Err(FieldSetError::UnionHaveNoFields {
                                name: field.name().to_string(),
                                ty: ctx[ctx.graph[id].name_id].to_string(),
                                path: value_path_to_string(ctx, value_path),
                            });
                        }
                    };
                    let definition_id = field_definition_ids
                        .into_iter()
                        .find(|id| ctx[ctx.graph[*id].name_id] == field.name())
                        .ok_or_else(|| FieldSetError::UnknownField {
                            name: field.name().to_string(),
                            ty: ctx.type_name(TypeRecord {
                                definition_id: parent_composite_type_id.into(),
                                wrapping: Default::default(),
                            }),
                            path: value_path_to_string(ctx, value_path),
                        })?;
                    out.push(convert_field(ctx, definition_id, field, value_path)?);
                    value_path.pop();
                }
                cynic_parser::executable::Selection::InlineFragment(fragment) => {
                    if let Some(type_condition) = fragment.type_condition() {
                        let definition_id = ctx
                            .graph
                            .type_definitions_ordered_by_name
                            .binary_search_by(|probe| {
                                match *probe {
                                    TypeDefinitionId::Scalar(id) => &ctx[ctx.graph[id].name_id],
                                    TypeDefinitionId::Object(id) => &ctx[ctx.graph[id].name_id],
                                    TypeDefinitionId::Interface(id) => &ctx[ctx.graph[id].name_id],
                                    TypeDefinitionId::Union(id) => &ctx[ctx.graph[id].name_id],
                                    TypeDefinitionId::Enum(id) => &ctx[ctx.graph[id].name_id],
                                    TypeDefinitionId::InputObject(id) => &ctx[ctx.graph[id].name_id],
                                }
                                .as_str()
                                .cmp(type_condition)
                            })
                            .map(|ix| ctx.graph.type_definitions_ordered_by_name[ix])
                            .map_err(|_| FieldSetError::UnknownType {
                                ty: type_condition.to_string(),
                                path: value_path_to_string(ctx, value_path),
                            })?;

                        let Some(composite_type_id) = definition_id.as_composite_type() else {
                            return Err(FieldSetError::NotAnOutputType {
                                ty: ctx.type_name(TypeRecord {
                                    definition_id,
                                    wrapping: Default::default(),
                                }),
                                path: value_path_to_string(ctx, value_path),
                            });
                        };

                        if is_disjoint(ctx, parent_composite_type_id, composite_type_id) {
                            return Err(FieldSetError::DisjointTypeCondition {
                                parent: ctx.type_name(TypeRecord {
                                    definition_id: parent_composite_type_id.into(),
                                    wrapping: Default::default(),
                                }),
                                name: ctx.type_name(TypeRecord {
                                    definition_id,
                                    wrapping: Default::default(),
                                }),
                                path: value_path_to_string(ctx, value_path),
                            });
                        }

                        if is_disjoint(ctx, parent_field_output, composite_type_id) {
                            continue;
                        }

                        stack.push((composite_type_id, fragment.selection_set()));
                    } else {
                        stack.push((parent_field_output, fragment.selection_set()));
                    }
                }
                cynic_parser::executable::Selection::FragmentSpread(_) => {
                    return Err(FieldSetError::CannotUseNamedFragments);
                }
            }
        }
    }

    Ok(FieldSetRecord::from(out))
}

fn convert_field(
    ctx: &mut GraphBuilder<'_>,
    definition_id: FieldDefinitionId,
    field: cynic_parser::executable::FieldSelection<'_>,
    value_path: &mut Vec<ValuePathSegment>,
) -> Result<FieldSetItemRecord, FieldSetError> {
    let subselection_record = if let Some(id) = ctx.graph[definition_id].ty_record.definition_id.as_composite_type() {
        if field.selection_set().len() == 0 {
            return Err(FieldSetError::LeafMustBeAScalarOrEnum {
                name: ctx[ctx.graph[definition_id].name_id].to_string(),
                ty: ctx.type_name(ctx.graph[definition_id].ty_record),
                path: value_path_to_string(ctx, value_path),
            });
        }
        value_path.push(ctx.graph[definition_id].name_id.into());
        let subselection = convert_selection_set(ctx, id, field.selection_set(), value_path)?;
        value_path.pop();
        subselection
    } else if field.selection_set().len() != 0 {
        return Err(FieldSetError::CannotHaveSelectionSet {
            name: ctx[ctx.graph[definition_id].name_id].to_string(),
            ty: ctx.type_name(ctx.graph[definition_id].ty_record),
            path: value_path_to_string(ctx, value_path),
        });
    } else {
        Default::default()
    };

    value_path.push(ctx.graph[definition_id].name_id.into());
    let field = SchemaFieldRecord {
        definition_id,
        sorted_argument_ids: convert_field_arguments(ctx, definition_id, field).map_err(|err| {
            FieldSetError::InputValueError {
                err,
                path: value_path_to_string(ctx, value_path),
            }
        })?,
    };
    value_path.pop();

    let field_id = ctx.selections.insert_field(field);
    Ok(FieldSetItemRecord {
        field_id,
        subselection_record,
    })
}

fn convert_field_arguments(
    ctx: &mut GraphBuilder<'_>,
    definition_id: FieldDefinitionId,
    field: cynic_parser::executable::FieldSelection<'_>,
) -> Result<IdRange<SchemaFieldArgumentId>, InputValueError> {
    let mut arguments = field.arguments().collect::<Vec<_>>();

    let start = ctx.selections.arguments.len();
    for argument_def_id in ctx.graph[definition_id].argument_ids {
        let InputValueDefinitionRecord {
            name_id,
            default_value_id,
            ty_record,
            ..
        } = ctx.graph[argument_def_id];
        if let Some(index) = arguments.iter().position(|argument| argument.name() == ctx[name_id]) {
            let argument = arguments.swap_remove(index);
            let value: cynic_parser::ConstValue<'_> = argument
                .value()
                .try_into()
                .map_err(|_| InputValueError::CannotUseVariables)?;
            let value_id = ctx.coerce_input_value(argument_def_id, value)?;
            ctx.selections.arguments.push(SchemaFieldArgumentRecord {
                definition_id: argument_def_id,
                value_id,
            });
        } else if let Some(value_id) = default_value_id {
            ctx.selections.arguments.push(SchemaFieldArgumentRecord {
                definition_id: argument_def_id,
                value_id,
            });
        } else if ty_record.wrapping.is_required() {
            return Err(InputValueError::MissingRequiredArgument(ctx.ctx[name_id].clone()));
        }
    }

    if let Some(first_unknown_argument) = arguments.first() {
        return Err(InputValueError::UnknownArgument(
            first_unknown_argument.name().to_string(),
        ));
    }

    let end = ctx.selections.arguments.len();
    ctx.selections.arguments[start..end].sort_unstable_by_key(|arg| arg.definition_id);
    Ok(IdRange::from(start..ctx.selections.arguments.len()))
}

pub(super) fn is_disjoint(ctx: &GraphBuilder<'_>, left: CompositeTypeId, right: CompositeTypeId) -> bool {
    let left: &[ObjectDefinitionId] = match &left {
        CompositeTypeId::Object(id) => std::array::from_ref(id),
        CompositeTypeId::Interface(id) => &ctx.graph[*id].possible_type_ids,
        CompositeTypeId::Union(id) => &ctx.graph[*id].possible_type_ids,
    };
    let right: &[ObjectDefinitionId] = match &right {
        CompositeTypeId::Object(id) => std::array::from_ref(id),
        CompositeTypeId::Interface(id) => &ctx.graph[*id].possible_type_ids,
        CompositeTypeId::Union(id) => &ctx.graph[*id].possible_type_ids,
    };

    let mut l = 0;
    let mut r = 0;
    while let (Some(left_id), Some(right_id)) = (left.get(l), right.get(r)) {
        match left_id.cmp(right_id) {
            std::cmp::Ordering::Less => l += 1,
            // At least one common object
            std::cmp::Ordering::Equal => return false,
            std::cmp::Ordering::Greater => r += 1,
        }
    }
    true
}
