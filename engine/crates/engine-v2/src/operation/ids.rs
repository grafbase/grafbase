use super::{
    Field, FieldArgument, LogicalPlan, LogicalPlanResponseBlueprint, Operation, OperationPlan, PreparedOperation,
    QueryModifier, ResponseBlueprint, SelectionSet, VariableDefinition,
};

id_newtypes::NonZeroU16! {
    Operation.fields[FieldId] => Field,
    Operation.selection_sets[SelectionSetId] => SelectionSet,
    Operation.variable_definitions[VariableDefinitionId] => VariableDefinition,
    Operation.field_arguments[FieldArgumentId] => FieldArgument,
    Operation.query_modifiers[QueryModifierId] => QueryModifier,
    Operation.query_modifiers_impacted_fields[ImpactedFieldId] => FieldId,
    OperationPlan.logical_plans[LogicalPlanId] => LogicalPlan | proxy(PreparedOperation.plan),
}

id_newtypes::index! {
    ResponseBlueprint.logical_plan_to_blueprint[LogicalPlanId] => LogicalPlanResponseBlueprint,
}
