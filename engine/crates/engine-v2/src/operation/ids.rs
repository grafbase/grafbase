use super::{Field, FieldArgument, LogicalPlan, Operation, QueryModifier, SelectionSet, VariableDefinition};

id_newtypes::NonZeroU16! {
    Operation.fields[FieldId] => Field,
    Operation.selection_sets[SelectionSetId] => SelectionSet,
    Operation.variable_definitions[VariableDefinitionId] => VariableDefinition,
    Operation.field_arguments[FieldArgumentId] => FieldArgument,
    Operation.logical_plans[LogicalPlanId] => LogicalPlan,
    Operation.query_modifiers[QueryModifierId] => QueryModifier,
    Operation.query_modifiers_impacted_fields[ImpactedFieldId] => FieldId,
}
