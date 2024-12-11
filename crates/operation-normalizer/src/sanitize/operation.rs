use cynic_parser::{common::OperationType, executable::OperationDefinition};

pub(super) fn sanitize(operation: &OperationDefinition<'_>, rendered: &mut String) {
    match operation.operation_type() {
        OperationType::Query => {
            rendered.push_str("query");
        }
        OperationType::Mutation => {
            rendered.push_str("mutation");
        }
        OperationType::Subscription => {
            rendered.push_str("subscription");
        }
    }

    if let Some(name) = operation.name() {
        rendered.push(' ');
        rendered.push_str(name);
    }

    super::variables::sanitize(operation.variable_definitions(), rendered);
    super::selection::sanitize(operation.selection_set(), rendered);
}
