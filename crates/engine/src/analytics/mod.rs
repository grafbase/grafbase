pub(crate) mod operation_name;
mod used_fields;

use schema::Schema;
use used_fields::UsedFields;

#[derive(Default)]
pub struct OperationAnalytics<'a> {
    /// Used fields, in the form of a iterator of (entity name, field name)
    pub used_fields: Option<UsedFields<'a>>,
    pub document_hash: [u8; 32],
}

pub struct ExecutedRequest<'a> {
    pub operation_name: Option<&'a str>,
    pub document: &'a str,
}

pub fn compute_post_execution_analytics<'a>(
    schema: &'a Schema,
    ExecutedRequest {
        operation_name,
        document,
    }: ExecutedRequest<'_>,
) -> OperationAnalytics<'a> {
    let Ok(parsed_operation) = crate::operation::parse(schema, operation_name, document) else {
        return Default::default();
    };

    let Ok(operation) = crate::operation::bind(schema, &parsed_operation) else {
        return Default::default();
    };

    let used_fields = Some(self::used_fields::compute(schema, &operation));
    let normalized_document = operation_normalizer::normalize(document, operation_name).unwrap_or_default();
    let document_hash = blake3::hash(normalized_document.as_bytes()).into();

    OperationAnalytics {
        used_fields,
        document_hash,
    }
}
