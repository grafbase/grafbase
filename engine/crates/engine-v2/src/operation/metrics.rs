use grafbase_telemetry::graphql::{GraphqlOperationAttributes, OperationName};

use super::parse::ParsedOperation;

/// Extracts the attributes from a given GraphQL operation and its document.
///
/// This function normalizes the provided GraphQL document using the operation's name and constructs
/// a `GraphqlOperationAttributes` object if successful. It determines the operation name based on
/// whether the name is explicitly provided in the operation or computed during execution for
/// consistency in tracing and metrics.
///
/// # Parameters
/// - `operation`: A reference to the `ParsedOperation` which contains metadata about the operation.
/// - `document`: A string slice that holds the GraphQL document to be normalized.
///
/// # Returns
/// An `Option<GraphqlOperationAttributes>` which is `Some` if attributes were successfully extracted,
/// or `None` if the normalization fails.
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
