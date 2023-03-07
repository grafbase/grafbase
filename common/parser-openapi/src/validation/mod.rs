//! Validates a parsed Graph against various rules

use crate::{
    graph::{InputValueKind, OpenApiGraph, QueryOperation},
    Error,
};

pub fn validate(graph: &OpenApiGraph) -> Result<(), Vec<Error>> {
    let errors = graph
        .query_operations()
        .into_iter()
        .filter_map(|operation| validate_operation(operation, graph).err())
        .flatten()
        .collect::<Vec<_>>();

    if !errors.is_empty() {
        return Err(errors);
    }
    Ok(())
}

fn validate_operation(operation: QueryOperation, graph: &OpenApiGraph) -> Result<(), Vec<Error>> {
    let operation_name = operation.name(graph).map(|name| name.to_string()).unwrap_or_default();

    let errors = operation
        .path_parameters(graph)
        .into_iter()
        .filter_map(|parameter| {
            let input_value = parameter.input_value(graph)?;
            if matches!(input_value.kind(graph), Some(InputValueKind::InputObject)) {
                return Some(Error::PathParameterIsObject(
                    parameter.name(graph).unwrap().to_string(),
                    operation_name.clone(),
                ));
            }
            if input_value.wrapping_type().contains_list() {
                return Some(Error::PathParameterIsList(
                    parameter.name(graph).unwrap().to_string(),
                    operation_name.clone(),
                ));
            }
            None
        })
        .collect::<Vec<_>>();

    if !errors.is_empty() {
        return Err(errors);
    }

    Ok(())
}
