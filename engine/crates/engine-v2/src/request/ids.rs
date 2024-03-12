use super::{
    BoundField, BoundFieldArgument, BoundFragment, BoundFragmentSpread, BoundInlineFragment, BoundSelectionSet,
    Operation, VariableDefinition,
};

id_newtypes::U16! {
    Operation.fields[BoundFieldId] => BoundField | unless "Too many fields",
    Operation.selection_sets[BoundSelectionSetId] => BoundSelectionSet | unless "Too many selection sets",
    Operation.fragments[BoundFragmentId] => BoundFragment | unless "Too many fragments",
    Operation.fragment_spreads[BoundFragmentSpreadId] => BoundFragmentSpread | unless "Too many fragment spreads",
    Operation.inline_fragments[BoundInlineFragmentId] => BoundInlineFragment | unless "Too many inline fragments",
    Operation.variable_definitions[VariableDefinitionId] => VariableDefinition | unless "Too many variables",
    Operation.field_arguments[BoundFieldArgumentId] => BoundFieldArgument | unless "Too many arguments",
}
