use std::num::NonZero;

#[id_derives::id]
pub struct LogicalPlanId(NonZero<u16>);

#[id_derives::id]
pub struct SelectionSetId(NonZero<u16>);

#[id_derives::id]
pub struct VariableDefinitionId(NonZero<u16>);

#[id_derives::id]
pub struct FieldId(NonZero<u16>);

#[id_derives::id]
pub struct FieldArgumentId(NonZero<u16>);

#[id_derives::id]
pub struct ResponseModifierId(NonZero<u16>);

#[id_derives::id]
pub struct ResponseModifierImpactedFieldId(NonZero<u16>);

#[id_derives::id]
pub struct QueryModifierId(NonZero<u16>);

#[id_derives::id]
pub struct QueryModifierImpactedFieldId(NonZero<u16>);
