use id_derives::Id;

use super::{Field, FieldArgument, Operation, QueryModifier, ResponseModifier, SelectionSet, VariableDefinition};

id_newtypes::NonZeroU16! {
    Operation.selection_sets[SelectionSetId] => SelectionSet,
    Operation.variable_definitions[VariableDefinitionId] => VariableDefinition,
    Operation.fields[FieldId] => Field,
    Operation.field_arguments[FieldArgumentId] => FieldArgument,
    Operation.response_modifiers[ResponseModifierId] => ResponseModifier,
    Operation.response_modifier_impacted_fields[ResponseModifierImpactedFieldId] => FieldId,
    Operation.query_modifiers[QueryModifierId] => QueryModifier,
    Operation.query_modifier_impacted_fields[QueryModifierImpactedFieldId] => FieldId,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, Id)]
pub struct LogicalPlanId(std::num::NonZero<u16>);
