//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-v2-codegen`
//! Source file: <engine-v2-codegen dir>/domain/operation_plan.graphql
use crate::plan::model::prelude::*;
use schema::{InputValueDefinition, InputValueDefinitionId};
use walker::Walk;

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type FieldArgument @meta(module: "field/argument") @indexed(id_size: "u16") {
///   definition: InputValueDefinition!
///   value_id: QueryInputValueId!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct FieldArgumentRecord {
    pub definition_id: InputValueDefinitionId,
    pub value_id: QueryInputValueId,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub(crate) struct FieldArgumentId(std::num::NonZero<u16>);

#[derive(Clone, Copy)]
pub(crate) struct FieldArgument<'a> {
    pub(in crate::plan::model) ctx: PlanContext<'a>,
    pub(in crate::plan::model) id: FieldArgumentId,
}

impl std::ops::Deref for FieldArgument<'_> {
    type Target = FieldArgumentRecord;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

#[allow(unused)]
impl<'a> FieldArgument<'a> {
    /// Prefer using Deref unless you need the 'a lifetime.
    #[allow(clippy::should_implement_trait)]
    pub(crate) fn as_ref(&self) -> &'a FieldArgumentRecord {
        &self.ctx.operation_plan[self.id]
    }
    pub(crate) fn id(&self) -> FieldArgumentId {
        self.id
    }
    pub(crate) fn definition(&self) -> InputValueDefinition<'a> {
        self.definition_id.walk(self.ctx.schema)
    }
}

impl<'a> Walk<PlanContext<'a>> for FieldArgumentId {
    type Walker<'w> = FieldArgument<'w> where 'a: 'w ;
    fn walk<'w>(self, ctx: PlanContext<'a>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        FieldArgument { ctx, id: self }
    }
}

impl std::fmt::Debug for FieldArgument<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FieldArgument")
            .field("definition", &self.definition())
            .field("value_id", &self.value_id)
            .finish()
    }
}
