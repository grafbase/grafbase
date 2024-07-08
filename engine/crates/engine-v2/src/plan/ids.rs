use super::{
    CollectedField, CollectedSelectionSet, ConditionalField, ConditionalSelectionSet, ExecutionPlan, OperationPlan,
};

id_newtypes::NonZeroU16! {
    OperationPlan.execution_plans[ExecutionPlanId] => ExecutionPlan,
    OperationPlan.conditional_fields[ConditionalFieldId] => ConditionalField,
    OperationPlan.conditional_selection_sets[ConditionalSelectionSetId] => ConditionalSelectionSet,
    OperationPlan.collected_selection_sets[CollectedSelectionSetId] => CollectedSelectionSet,
    OperationPlan.collected_fields[CollectedFieldId] => CollectedField,
}

id_newtypes::NonZeroU16! {
    ResponseObjectSetId,
}
