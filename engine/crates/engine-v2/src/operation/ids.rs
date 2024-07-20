use super::{
    Field, FieldArgument, LogicalPlan, LogicalPlanResponseBlueprint, Operation, OperationPlan, PreparedOperation,
    QueryModifier, ResponseBlueprint, ResponseModifier, ResponseModifierRule, SelectionSet, VariableDefinition,
};

id_newtypes::NonZeroU16! {
    Operation.selection_sets[SelectionSetId] => SelectionSet,
    Operation.variable_definitions[VariableDefinitionId] => VariableDefinition,
    Operation.fields[FieldId] => Field,
    Operation.field_arguments[FieldArgumentId] => FieldArgument,
    Operation.fields_subject_to_response_modifier_rules[SubjectToResponseModifierRuleId] => ResponseModifierRuleId,
    Operation.response_modifier_rules[ResponseModifierRuleId] => ResponseModifierRule,
    Operation.query_modifiers[QueryModifierId] => QueryModifier,
    Operation.query_modifiers_impacted_fields[ImpactedFieldId] => FieldId,
    OperationPlan.logical_plans[LogicalPlanId] => LogicalPlan | proxy(PreparedOperation.plan),
    ResponseBlueprint.logical_plan_response_modifiers[ResponseModifierId] => ResponseModifier | proxy(PreparedOperation.response_blueprint),
}

id_newtypes::index! {
    ResponseBlueprint.logical_plan_to_blueprint[LogicalPlanId] => LogicalPlanResponseBlueprint,
}
