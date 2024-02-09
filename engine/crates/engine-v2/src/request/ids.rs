use super::{
    BoundField, BoundFieldArguments, BoundFragment, BoundFragmentSpread, BoundInlineFragment, BoundSelectionSet,
    Operation, VariableDefinition,
};

crate::utils::id_newtypes! {
    Operation.fields[BoundFieldId] => BoundField unless "Too many fields",
    Operation.selection_sets[BoundSelectionSetId] => BoundSelectionSet unless "Too many selection sets",
    Operation.fragments[BoundFragmentId] => BoundFragment unless "Too many fragments",
    Operation.fragment_spreads[BoundFragmentSpreadId] => BoundFragmentSpread unless "Too many fragment spreads",
    Operation.inline_fragments[BoundInlineFragmentId] => BoundInlineFragment unless "Too many inline fragments",
    Operation.variable_definitions[VariableDefinitionId] => VariableDefinition unless "Too many variables",
    Operation.field_arguments[BoundFieldArgumentsId] => BoundFieldArguments unless "Too many arguments",
}
