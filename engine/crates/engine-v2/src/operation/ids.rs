use super::{Condition, Field, FieldArgument, Operation, Plan, SelectionSet, VariableDefinition};

id_newtypes::NonZeroU16! {
    Operation.fields[FieldId] => Field,
    Operation.selection_sets[SelectionSetId] => SelectionSet,
    Operation.variable_definitions[VariableDefinitionId] => VariableDefinition,
    Operation.field_arguments[FieldArgumentId] => FieldArgument,
    Operation.conditions[ConditionId] => Condition,
    Operation.plans[PlanId] => Plan,
}
