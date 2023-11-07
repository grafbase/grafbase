use engine_parser::{Pos, Positioned};

use engine::{ServerError, ServerResult};

use graph::{FieldId, FieldTypeId, Graph, InterfaceId, ObjectId, StringId, UnionId};

#[allow(dead_code)]
pub struct VariableId(usize);

#[allow(dead_code)]
pub enum TypeCondition {
    Interface(InterfaceId),
    Object(ObjectId),
    Union(UnionId),
}

#[allow(dead_code)]
pub struct OperationDefinition {
    ty: engine_parser::types::OperationType,
    selection_set: SelectionSet,
}

pub struct SelectionSet {
    pub items: Vec<Selection>,
}

pub enum Selection {
    Field(Field),
}

#[allow(dead_code)]
pub struct Field {
    pos: Pos,
    id: FieldId,
    alias: Option<Alias>,
    arguments: Vec<Argument>,
    selection_set: SelectionSet,
}

#[allow(dead_code)]
pub struct Alias {
    value: String,
    pos: Pos,
}

#[allow(dead_code)]
pub struct Argument {
    name_pos: Pos,
    name: StringId,
    type_id: FieldTypeId,
    value_pos: Pos,
    value: engine_value::Value,
}

#[allow(dead_code)]
pub fn bind_operation(
    operation: engine_parser::types::OperationDefinition,
    graph: &Graph,
) -> ServerResult<OperationDefinition> {
    Ok(OperationDefinition {
        ty: operation.ty,
        selection_set: bind_selection_set(
            graph,
            match operation.ty {
                engine_parser::types::OperationType::Query => graph.query_fields().collect(),
                engine_parser::types::OperationType::Mutation => graph.mutation_fields().collect(),
                engine_parser::types::OperationType::Subscription => todo!(),
            },
            operation.selection_set,
        )?,
    })
}

fn bind_selection_set(
    graph: &Graph,
    field_ids: Vec<FieldId>,
    selection_set: Positioned<engine_parser::types::SelectionSet>,
) -> ServerResult<SelectionSet> {
    let Positioned {
        pos: _,
        node: selection_set,
    } = selection_set;
    let items = selection_set
        .items
        .into_iter()
        .map(|Positioned { node: selection, .. }| match selection {
            engine_parser::types::Selection::Field(Positioned { pos, node: field }) => {
                let name = field.name.node.to_string();
                let (id, graph_field) = field_ids
                    .iter()
                    .find_map(|id| {
                        let field = &graph[*id];
                        if graph[field.name] == name {
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
                            match graph_field.arguments.iter().find(|arg| graph[arg.name] == name) {
                                Some(graph::FieldArgument { name, type_id }) => Ok(Argument {
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

                let selection_set = bind_selection_set(
                    graph,
                    match graph[graph_field.field_type_id].kind {
                        graph::Definition::Object(object_id) => graph.object_fields(object_id).collect(),
                        graph::Definition::Interface(interface_id) => graph.interface_fields(interface_id).collect(),
                        _ => {
                            return Err(ServerError::new(
                                format!("Field {name} does not have any fields."),
                                Some(pos),
                            ));
                        }
                    },
                    field.selection_set,
                )?;

                Ok(Selection::Field(Field {
                    pos,
                    id: *id,
                    alias: field.alias.map(|Positioned { pos, node }| Alias {
                        value: node.to_string(),
                        pos,
                    }),
                    arguments,
                    selection_set,
                }))
            }
            engine_parser::types::Selection::FragmentSpread(_) => todo!(),
            engine_parser::types::Selection::InlineFragment(_) => todo!(),
        })
        .collect::<ServerResult<_>>()?;
    Ok(SelectionSet { items })
}
