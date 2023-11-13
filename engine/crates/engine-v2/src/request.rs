use engine::{ServerError, ServerResult};
pub use engine_parser::types::OperationType;
use engine_parser::Positioned;
use schema::{FieldId, Schema};

use crate::response::{Argument, ResponseFields, ResponseFieldsBuilder, Selection, SelectionSet};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct VariableId(usize);

pub struct OperationDefinition {
    pub ty: OperationType,
    pub selection_set: SelectionSet,
    pub response_edges_builder: ResponseFieldsBuilder,
}

pub struct OperationBinder<'a> {
    response_edges_builder: ResponseFieldsBuilder,
    schema: &'a Schema,
}

impl<'a> OperationBinder<'a> {
    pub fn new(schema: &'a Schema) -> Self {
        Self {
            response_edges_builder: ResponseFields::builder(),
            schema,
        }
    }

    pub fn bind(mut self, operation: engine_parser::types::OperationDefinition) -> ServerResult<OperationDefinition> {
        let root_object_id = match operation.ty {
            OperationType::Query => self.schema.root_operation_types.query,
            OperationType::Mutation => self
                .schema
                .root_operation_types
                .mutation
                .expect("Mutation operation type not supported by schema."),
            OperationType::Subscription => self
                .schema
                .root_operation_types
                .subscription
                .expect("Subscription operation type not supported by schema."),
        };
        let selection_set = self.bind_selection_set(
            self.schema.object_fields(root_object_id).collect(),
            operation.selection_set,
        )?;
        Ok(OperationDefinition {
            ty: operation.ty,
            selection_set,
            response_edges_builder: self.response_edges_builder,
        })
    }

    fn bind_selection_set(
        &mut self,
        field_ids: Vec<FieldId>,
        selection_set: Positioned<engine_parser::types::SelectionSet>,
    ) -> ServerResult<SelectionSet> {
        let Positioned {
            pos: _,
            node: selection_set,
        } = selection_set;
        selection_set
            .items
            .into_iter()
            .map(|Positioned { node: selection, .. }| match selection {
                engine_parser::types::Selection::Field(Positioned { pos, node: field }) => {
                    let name = field.name.node.to_string();
                    let (&field_id, schema_field) = field_ids
                        .iter()
                        .find_map(|id| {
                            let field = &self.schema[*id];
                            if self.schema[field.name] == name {
                                Some((id, field))
                            } else {
                                None
                            }
                        })
                        .ok_or_else(|| ServerError::new(format!("Field {name} does not exist"), Some(pos)))?;

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
                                match schema_field.arguments.iter().find(|arg| self.schema[arg.name] == name) {
                                    Some(schema::FieldArgument { name, type_id }) => Ok(Argument {
                                        name_pos,
                                        name: *name,
                                        type_id: *type_id,
                                        value_pos,
                                        value,
                                    }),
                                    None => Err(ServerError::new(
                                        format!("Argument {name} does not exist."),
                                        Some(name_pos),
                                    )),
                                }
                            },
                        )
                        .collect::<ServerResult<_>>()?;

                    let subselection = if field.selection_set.node.items.is_empty() {
                        SelectionSet::empty()
                    } else {
                        self.bind_selection_set(
                            match self.schema[schema_field.field_type_id].kind {
                                schema::Definition::Object(object_id) => self.schema.object_fields(object_id).collect(),
                                schema::Definition::Interface(interface_id) => {
                                    self.schema.interface_fields(interface_id).collect()
                                }
                                _ => {
                                    return Err(ServerError::new(
                                        format!("Field {name} does not have any fields."),
                                        Some(pos),
                                    ));
                                }
                            },
                            field.selection_set,
                        )?
                    };
                    Ok(Selection {
                        field: self.response_edges_builder.push_field(
                            &field
                                .alias
                                .map(|Positioned { node, .. }| node.to_string())
                                .unwrap_or(name),
                            pos,
                            field_id,
                            None,
                            arguments,
                        ),
                        subselection,
                    })
                }
                engine_parser::types::Selection::FragmentSpread(_) => todo!(),
                engine_parser::types::Selection::InlineFragment(_) => todo!(),
            })
            .collect()
    }
}
