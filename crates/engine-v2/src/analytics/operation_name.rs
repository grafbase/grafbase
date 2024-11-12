use crate::operation::ParsedOperation;

pub(crate) fn compute(operation: &ParsedOperation) -> Option<String> {
    engine_parser::find_first_field_name(&operation.fragments, &operation.definition.selection_set)
}
