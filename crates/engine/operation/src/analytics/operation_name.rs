use cynic_parser::executable::{Iter, Selection};

use crate::parse::ParsedOperation;

pub(crate) fn compute_operation_name(operation: &ParsedOperation) -> Option<String> {
    fn first_field_in_set(mut selection_set: Iter<'_, Selection<'_>>) -> Option<String> {
        selection_set.find_map(|selection| match &selection {
            Selection::Field(field) => Some(field.alias().unwrap_or(field.name()).to_string()),
            Selection::InlineFragment(fragment) => first_field_in_set(fragment.selection_set()),
            Selection::FragmentSpread(spread) => first_field_in_set(spread.fragment()?.selection_set()),
        })
    }
    first_field_in_set(operation.operation().selection_set())
}
