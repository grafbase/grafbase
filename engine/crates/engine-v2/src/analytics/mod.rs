mod operation_name;
mod used_fields;

use schema::Schema;
use used_fields::UsedFields;

#[derive(Default)]
pub struct OperationAnalytics<'a> {
    /// Generated operation name if none was provided.
    pub generated_operation_name: Option<String>,
    /// Used fields, in the form of a iterator of (entity name, field name)
    pub used_fields: Option<UsedFields<'a>>,
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
    let Ok(parsed_operation) = crate::operation::parse_operation(operation_name, document) else {
        return Default::default();
    };
    let operation_name = if operation_name.is_none() {
        self::operation_name::compute(&parsed_operation)
    } else {
        None
    };

    let Ok(operation) = crate::operation::bind_operation(schema, parsed_operation) else {
        return OperationAnalytics {
            generated_operation_name: operation_name,
            ..Default::default()
        };
    };
    let used_fields = Some(self::used_fields::compute(schema, &operation));

    OperationAnalytics {
        generated_operation_name: operation_name,
        used_fields,
    }
}
