use grafbase_telemetry::graphql::{OperationName, OperationType};

use crate::prepare::CachedOperationAttributes;

use super::parse::ParsedOperation;

pub(crate) fn extract_attributes(operation: &ParsedOperation, document: &str) -> Option<CachedOperationAttributes> {
    operation_normalizer::normalize(document, operation.name.as_deref())
        .ok()
        .map(|sanitized_query| CachedOperationAttributes {
            ty: convert_operation_type(operation.operation().operation_type()),
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

fn convert_operation_type(ty: cynic_parser::common::OperationType) -> OperationType {
    match ty {
        cynic_parser::common::OperationType::Query => OperationType::Query,
        cynic_parser::common::OperationType::Mutation => OperationType::Mutation,
        cynic_parser::common::OperationType::Subscription => OperationType::Subscription,
    }
}
