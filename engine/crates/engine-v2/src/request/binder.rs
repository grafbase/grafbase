use engine::{ServerError, ServerResult};
pub use engine_parser::types::OperationType;
use engine_parser::{types::OperationDefinition, Positioned};
use schema::{FieldId, Schema};

use super::{
    fields::OperationField, OperationArgument, OperationFieldsBuilder, OperationSelection, OperationSelectionSet,
};

pub struct OperationBinder<'a, 'b> {
    pub(super) schema: &'a Schema,
    pub(super) fields: &'a mut OperationFieldsBuilder<'b>,
}

impl<'a, 'b> OperationBinder<'a, 'b> {
    pub fn bind(mut self, operation_definition: OperationDefinition) -> ServerResult<OperationSelectionSet> {
        let root_object_id = match operation_definition.ty {
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
        self.bind_selection_set(
            self.schema.object_fields(root_object_id).collect(),
            operation_definition.selection_set,
        )
    }

    fn bind_selection_set(
        &mut self,
        field_ids: Vec<FieldId>,
        selection_set: Positioned<engine_parser::types::SelectionSet>,
    ) -> ServerResult<OperationSelectionSet> {
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
                                    Some(schema::FieldArgument { name, type_id }) => Ok(OperationArgument {
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
                        OperationSelectionSet::empty()
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
                    let response_field_id = self.fields.push(OperationField {
                        name: &field
                            .alias
                            .map(|Positioned { node, .. }| node.to_string())
                            .unwrap_or(name),
                        position,
                        pos,
                        field_id,
                        type_condition: None,
                        arguments,
                    });
                    Ok(OperationSelection {
                        op_field_id: response_field_id,
                        name: self.fields[response_field_id].name,
                        subselection,
                    })
                }
                engine_parser::types::Selection::FragmentSpread(_) => todo!(),
                engine_parser::types::Selection::InlineFragment(_) => todo!(),
            })
            .collect()
    }
}
