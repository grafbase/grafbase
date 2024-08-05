use std::num::NonZero;

#[id_derives::id]
pub struct ExecutionPlanId(NonZero<u16>);

#[id_derives::id]
pub struct ErrorId(NonZero<u16>);

#[id_derives::id]
pub struct ResponseModifierExecutorId(NonZero<u16>);
