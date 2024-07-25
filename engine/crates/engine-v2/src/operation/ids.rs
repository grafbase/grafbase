use super::{
    Field, FieldArgument, LogicalPlan, LogicalPlanResponseBlueprint, Operation, OperationPlan, PreparedOperation,
    QueryModifier, ResponseBlueprint, ResponseModifier, SelectionSet, VariableDefinition,
};

id_newtypes::NonZeroU16! {
    Operation.selection_sets[SelectionSetId] => SelectionSet,
    Operation.variable_definitions[VariableDefinitionId] => VariableDefinition,
    Operation.fields[FieldId] => Field,
    Operation.field_arguments[FieldArgumentId] => FieldArgument,
    Operation.response_modifiers[ResponseModifierId] => ResponseModifier,
    Operation.response_modifier_impacted_fields[ResponseModifierImpactedFieldId] => FieldId,
    Operation.query_modifiers[QueryModifierId] => QueryModifier,
    Operation.query_modifier_impacted_fields[QueryModifierImpactedFieldId] => FieldId,
    OperationPlan.logical_plans[LogicalPlanId] => LogicalPlan | proxy(PreparedOperation.plan),
}

id_newtypes::index! {
    OperationPlan.field_to_logical_plan_id[FieldId] => LogicalPlanId,
    ResponseBlueprint.logical_plan_to_blueprint[LogicalPlanId] => LogicalPlanResponseBlueprint,
}
