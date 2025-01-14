use crate::OperationAttributes;

use super::parse::ParsedOperation;
use grafbase_telemetry::graphql::{OperationName, OperationType};

pub(crate) fn extract_attributes(operation: &ParsedOperation) -> OperationAttributes {
    let ty = convert_operation_type(operation.operation().operation_type());

    let name = if let Some(name) = operation.name.clone() {
        OperationName::Original(name)
    } else if let Some(name) = crate::analytics::compute_operation_name(operation) {
        // We have to compute the name during the execution to ensure traces and metrics are
        // consistent with each other. For metrics it can be computed later efficiently, but
        // not for spans.
        OperationName::Computed(name)
    } else {
        OperationName::Unknown
    };

    let sanitized_query = operation_normalizer::sanitize(operation.document()).into();

    OperationAttributes {
        ty,
        name,
        sanitized_query,
    }
}

fn convert_operation_type(ty: cynic_parser::common::OperationType) -> OperationType {
    match ty {
        cynic_parser::common::OperationType::Query => OperationType::Query,
        cynic_parser::common::OperationType::Mutation => OperationType::Mutation,
        cynic_parser::common::OperationType::Subscription => OperationType::Subscription,
    }
}
