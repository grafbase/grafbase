use grafbase_telemetry::graphql::{GraphqlOperationAttributes, OperationName};

use super::parse::ParsedOperation;

pub(super) fn extract_attributes(operation: &ParsedOperation, document: &str) -> Option<GraphqlOperationAttributes> {
    operation_normalizer::normalize(document, operation.name.as_deref())
        .ok()
        .map(|sanitized_query| GraphqlOperationAttributes {
            ty: operation.definition.ty.into(),
            name: if let Some(name) = operation.name.clone() {
                OperationName::Original(name)
            } else if let Some(name) = crate::analytics::operation_name::compute(operation) {
                // We have to compute the name during the execution to ensure traces and metrics are
                // consistent with each other. For metrics it can be computed later efficiently, but
                // not for spans.
                OperationName::Computed(name)
            } else {
                OperationName::Unknown
            },
            sanitized_query: sanitized_query.into(),
        })
}
