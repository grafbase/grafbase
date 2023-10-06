//! Validates a parsed Graph against various rules

use engine::registry::resolvers::http::QueryParameterEncodingStyle;

use crate::{
    graph::{InputValueKind, OpenApiGraph, Operation},
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

fn validate_operation(operation: Operation, graph: &OpenApiGraph) -> Result<(), Vec<Error>> {
    let operation_name = operation.name(graph).map(|name| name.to_string()).unwrap_or_default();

    let mut errors = operation
        .path_parameters(graph)
        .into_iter()
        .filter_map(|parameter| {
            let input_value = parameter.input_value(graph)?;
            if matches!(input_value.kind(graph), Some(InputValueKind::InputObject)) {
                return Some(Error::PathParameterIsObject(
                    parameter.openapi_name(graph).to_string(),
                    operation_name.clone(),
                ));
            }
            if input_value.wrapping_type().contains_list() {
                return Some(Error::PathParameterIsList(
                    parameter.openapi_name(graph).to_string(),
                    operation_name.clone(),
                ));
            }
            None
        })
        .collect::<Vec<_>>();

    errors.extend(operation.query_parameters(graph).into_iter().filter_map(|parameter| {
        let input_value = parameter.input_value(graph)?;
        if parameter.encoding_style(graph) == Some(QueryParameterEncodingStyle::DeepObject) {
            // DeepObject encoding allows nested objects because stripe uses them.
            // The OAI spec says nested objects or lists are undefined behaviour even
            // for DeepObject, but since stripe uses them we're kind of stuffed.
            return None;
        }

        // Make sure we don't have any nested lists or objects as we can't really encode them.
        if input_value.wrapping_type().contains_list() {
            if matches!(input_value.kind(graph), Some(InputValueKind::InputObject)) {
                // We don't support encoding nested objects inside query strings so this is an error.
                return Some(Error::ObjectNestedInsideListQueryParamter(
                    parameter.openapi_name(graph).to_owned(),
                    operation_name.clone(),
                ));
            }
        } else if matches!(input_value.kind(graph), Some(InputValueKind::InputObject)) {
            let object = input_value.as_input_object(graph)?;
            for field in object.fields(graph) {
                if field.value_type.wrapping_type().contains_list() {
                    return Some(Error::ListNestedInsideObjectQueryParameter(
                        parameter.openapi_name(graph).to_owned(),
                        operation_name.clone(),
                    ));
                }
                if !matches!(field.value_type.kind(graph), Some(InputValueKind::Scalar)) {
                    return Some(Error::NonScalarNestedInsideObjectQueryParameter(
                        parameter.openapi_name(graph).to_owned(),
                        operation_name.clone(),
                    ));
                }
            }
        }
        None
    }));

    if !errors.is_empty() {
        return Err(errors);
    }

    Ok(())
}
