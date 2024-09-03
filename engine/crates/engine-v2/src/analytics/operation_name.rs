use crate::operation::ParsedOperation;

pub(super) fn compute(operation: &ParsedOperation) -> Option<String> {
    engine_parser::find_first_field_name(&operation.fragments, &operation.definition.selection_set)
}
