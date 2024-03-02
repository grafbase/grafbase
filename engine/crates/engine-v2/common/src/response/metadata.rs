// TODO: remove me once we move to tail workers for logs
#[derive(Default, Debug)]
pub struct ExecutionMetadata {
    pub operation_name: Option<String>,
    pub operation_type: Option<common_types::OperationType>,
    pub has_errors: Option<bool>,
}
