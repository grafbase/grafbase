//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-codegen`
//! Source file: <engine-codegen dir>/domain/operation_plan.graphql
use crate::prepare::operation_plan::model::{
    generated::{Executable, ExecutableId},
    prelude::*,
};
use walker::{Iter, Walk};

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type Plan @indexed(id_size: "u16") @meta(module: "plan") {
///   query_partition_id: QueryPartitionId!
///   required_fields: RequiredFieldSet!
///   resolver: Resolver!
///   parent_count: usize!
///   children: [Executable!]!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct PlanRecord {
    pub query_partition_id: QueryPartitionId,
    pub required_fields_record: RequiredFieldSetRecord,
    pub resolver: Resolver,
    pub parent_count: usize,
    pub children_ids: Vec<ExecutableId>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub(crate) struct PlanId(std::num::NonZero<u16>);

#[derive(Clone, Copy)]
pub(crate) struct Plan<'a> {
    pub(in crate::prepare::operation_plan::model) ctx: OperationPlanContext<'a>,
    pub(crate) id: PlanId,
}

impl std::ops::Deref for Plan<'_> {
    type Target = PlanRecord;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

#[allow(unused)]
impl<'a> Plan<'a> {
    /// Prefer using Deref unless you need the 'a lifetime.
    #[allow(clippy::should_implement_trait)]
    pub(crate) fn as_ref(&self) -> &'a PlanRecord {
        &self.ctx.plan[self.id]
    }
    pub(crate) fn required_fields(&self) -> RequiredFieldSet<'a> {
        self.as_ref().required_fields_record.walk(self.ctx)
    }
    pub(crate) fn children(&self) -> impl Iter<Item = Executable<'a>> + 'a {
        self.as_ref().children_ids.walk(self.ctx)
    }
}

impl<'a> Walk<OperationPlanContext<'a>> for PlanId {
    type Walker<'w>
        = Plan<'w>
    where
        'a: 'w;
    fn walk<'w>(self, ctx: impl Into<OperationPlanContext<'a>>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        Plan {
            ctx: ctx.into(),
            id: self,
        }
    }
}

impl std::fmt::Debug for Plan<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Plan")
            .field("query_partition_id", &self.query_partition_id)
            .field("required_fields", &self.required_fields())
            .field("resolver", &self.resolver)
            .field("parent_count", &self.parent_count)
            .field("children", &self.children())
            .finish()
    }
}
