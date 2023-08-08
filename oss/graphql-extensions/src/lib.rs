use dynaql::parser::types::{OperationDefinition, OperationType};

pub mod authorization;
pub mod runtime_log;

fn is_operation_introspection(operation: &OperationDefinition) -> bool {
    use dynaql::parser::types::Selection;
    operation.ty == OperationType::Query
        && operation
            .selection_set
            .node
            .items
            .iter()
            // If field name starts with `__` it is part of introspection system, see http://spec.graphql.org/October2021/#sec-Names.Reserved-Names
            .all(|item| {
                matches!(
                &item.node,
                Selection::Field(field) if field.node.name.node.starts_with("__"))
            })
}
