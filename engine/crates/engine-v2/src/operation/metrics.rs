use grafbase_telemetry::metrics::OperationMetricsAttributes;

use super::parse::ParsedOperation;

pub(super) fn prepare_metrics_attributes(
    operation: &ParsedOperation,
    document: &str,
) -> Option<OperationMetricsAttributes> {
    operation_normalizer::normalize(document, operation.name.as_deref())
        .ok()
        .map(|sanitized_query| OperationMetricsAttributes {
            ty: operation.definition.ty.into(),
            name: operation.name.clone(),
            sanitized_query: sanitized_query.into(),
        })
}
