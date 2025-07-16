//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-codegen`
//! Source file: <engine-codegen dir>/domain/query_plan.graphql
use crate::prepare::cached::query_plan::{PlanValueRecord, prelude::*};
use schema::{InputValueDefinition, InputValueDefinitionId};
#[allow(unused_imports)]
use walker::{Iter, Walk};

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type PartitionFieldArgument @meta(module: "argument") {
///   definition: InputValueDefinition!
///   value_record: PlanValueRecord!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub(crate) struct PartitionFieldArgumentRecord {
    pub definition_id: InputValueDefinitionId,
    pub value_record: PlanValueRecord,
}

#[derive(Clone, Copy)]
pub(crate) struct PartitionFieldArgument<'a> {
    pub(in crate::prepare::cached::query_plan) ctx: CachedOperationContext<'a>,
    pub(in crate::prepare::cached::query_plan) ref_: &'a PartitionFieldArgumentRecord,
}

impl std::ops::Deref for PartitionFieldArgument<'_> {
    type Target = PartitionFieldArgumentRecord;
    fn deref(&self) -> &Self::Target {
        self.ref_
    }
}

#[allow(unused)]
impl<'a> PartitionFieldArgument<'a> {
    #[allow(clippy::should_implement_trait)]
    pub(crate) fn as_ref(&self) -> &'a PartitionFieldArgumentRecord {
        self.ref_
    }
    pub(crate) fn definition(&self) -> InputValueDefinition<'a> {
        self.definition_id.walk(self.ctx)
    }
}

impl<'a> Walk<CachedOperationContext<'a>> for &PartitionFieldArgumentRecord {
    type Walker<'w>
        = PartitionFieldArgument<'w>
    where
        Self: 'w,
        'a: 'w;
    fn walk<'w>(self, ctx: impl Into<CachedOperationContext<'a>>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        PartitionFieldArgument {
            ctx: ctx.into(),
            ref_: self,
        }
    }
}

impl std::fmt::Debug for PartitionFieldArgument<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PartitionFieldArgument")
            .field("definition", &self.definition())
            .field("value_record", &self.value_record)
            .finish()
    }
}
