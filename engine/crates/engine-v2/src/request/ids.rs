use super::{
    BoundAnyFieldDefinition, BoundField, BoundFragmentDefinition, BoundSelectionSet, Operation, VariableDefinition,
};

crate::utils::id_newtypes! {
    Operation.fields[BoundFieldId] => BoundField unless "Too many fields",
    Operation.selection_sets[BoundSelectionSetId] => BoundSelectionSet unless "Too many selection sets",
    Operation.field_definitions[BoundAnyFieldDefinitionId] => BoundAnyFieldDefinition unless "Too many fields",
    Operation.fragment_definitions[BoundFragmentDefinitionId] => BoundFragmentDefinition unless "Too many fragments",
    Operation.variable_definitions[VariableDefinitionId] => VariableDefinition unless "Too many variables",
}
