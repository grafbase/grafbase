pub use engine_parser::types::OperationType;
use engine_parser::{types::OperationDefinition, Positioned};
use schema::{ObjectId, Schema};

use super::{OperationField, OperationFieldArgument, OperationFieldId, OperationSelection, OperationSelectionSet, Pos};
use crate::{execution::Strings, response::GraphqlError};

#[derive(thiserror::Error, Debug)]
pub enum BindError {
    #[error("{container} does not have a field named '{name}'")]
    UnknownField { container: String, name: String, pos: Pos },
    #[error("Field {field} does not have an argument named '{name}'")]
    UnknownFieldArgument { field: String, name: String, pos: Pos },
    #[error("Field {name} cannot have a field selection, it's a {ty}. Only unions, interfaces and objects can.")]
    CannotHaveFieldSelection { name: String, ty: String, pos: Pos },
    #[error("Mutations are not defined on this schema.")]
    NoMutationDefined,
    #[error("Subscriptions are not defined on this schema.")]
    NoSubscriptionDefined,
}

impl From<BindError> for GraphqlError {
    fn from(err: BindError) -> Self {
        let locations = match err {
            BindError::UnknownField { pos, .. }
            | BindError::UnknownFieldArgument { pos, .. }
            | BindError::CannotHaveFieldSelection { pos, .. } => vec![pos],
            _ => vec![],
        };
        GraphqlError {
            message: err.to_string(),
            locations,
            path: vec![],
        }
    }
}

impl BindError {
    pub fn pos(&self) -> Option<Pos> {
        match self {
            Self::UnknownField { pos, .. }
            | Self::UnknownFieldArgument { pos, .. }
            | Self::CannotHaveFieldSelection { pos, .. } => Some(*pos),
            _ => None,
        }
    }
}

pub type BindResult<T> = Result<T, BindError>;

pub struct Binder<'a> {
    schema: &'a Schema,
    pub(super) fields: Vec<OperationField>,
    pub(super) strings: Strings,
}

impl<'a> Binder<'a> {
    pub fn new(schema: &'a Schema) -> Self {
        Self {
            schema,
            fields: Vec::new(),
            strings: Strings::new(),
        }
    }

    pub fn bind(&mut self, operation_definition: OperationDefinition) -> BindResult<OperationSelectionSet> {
        let object = match operation_definition.ty {
            OperationType::Query => self.schema.root_operation_types.query,
            OperationType::Mutation => self
                .schema
                .root_operation_types
                .mutation
                .ok_or(BindError::NoMutationDefined)?,
            OperationType::Subscription => self
                .schema
                .root_operation_types
                .subscription
                .ok_or(BindError::NoSubscriptionDefined)?,
        };
        self.bind_object_selection_set(object, operation_definition.selection_set)
    }

    fn bind_object_selection_set(
        &mut self,
        object_id: ObjectId,
        selection_set: Positioned<engine_parser::types::SelectionSet>,
    ) -> BindResult<OperationSelectionSet> {
        let Positioned {
            pos: _,
            node: selection_set,
        } = selection_set;
        // Keeping the original ordering
        selection_set
            .items
            .into_iter()
            .enumerate()
            .map(|(position, Positioned { node: selection, .. })| match selection {
                engine_parser::types::Selection::Field(Positioned { pos, node: field }) => {
                    let name = field.name.node.to_string();
                    let schema_field = self
                        .schema
                        .object_field_by_name(object_id, &name)
                        .map(|field_id| self.schema.default_walker().walk(field_id))
                        .ok_or_else(|| BindError::UnknownField {
                            container: self.schema[self.schema[object_id].name].to_string(),
                            name: name.clone(),
                            pos,
                        })?;

                    let arguments = field
                        .arguments
                        .into_iter()
                        .map(
                            |(
                                Positioned {
                                    pos: name_pos,
                                    node: name,
                                },
                                Positioned {
                                    pos: value_pos,
                                    node: value,
                                },
                            )| {
                                let name = name.to_string();
                                schema_field
                                    .argument_by_name(&name)
                                    .map(|input_value| OperationFieldArgument {
                                        name_pos,
                                        input_value_id: input_value.id,
                                        value_pos,
                                        value,
                                    })
                                    .ok_or_else(|| BindError::UnknownFieldArgument {
                                        field: schema_field.name().to_string(),
                                        name,
                                        pos: name_pos,
                                    })
                            },
                        )
                        .collect::<BindResult<_>>()?;

                    let subselection = if field.selection_set.node.items.is_empty() {
                        OperationSelectionSet::empty()
                    } else {
                        match schema_field.ty().inner {
                            schema::Definition::Object(object_id) => {
                                self.bind_object_selection_set(object_id, field.selection_set)?
                            }
                            schema::Definition::Interface(_interface_id) => todo!(),
                            schema::Definition::Union(_union_id) => todo!(),
                            _ => {
                                return Err(BindError::CannotHaveFieldSelection {
                                    name,
                                    ty: schema_field.ty().inner().name().to_string(),
                                    pos,
                                })
                            }
                        }
                    };
                    let name = &field
                        .alias
                        .map(|Positioned { node, .. }| node.to_string())
                        .unwrap_or(name);
                    self.fields.push(OperationField {
                        name: self.strings.get_or_intern(name),
                        position,
                        pos,
                        field_id: schema_field.id,
                        type_condition: None,
                        arguments,
                    });
                    Ok(OperationSelection {
                        operation_field_id: OperationFieldId((self.fields.len() - 1) as u32),
                        subselection,
                    })
                }
                engine_parser::types::Selection::FragmentSpread(_) => todo!(),
                engine_parser::types::Selection::InlineFragment(_) => todo!(),
            })
            .collect()
    }
}
