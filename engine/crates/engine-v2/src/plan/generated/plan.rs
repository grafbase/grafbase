//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-v2-codegen`
//! Source file: <engine-v2-codegen dir>/domain/operation_plan.graphql
use crate::plan::{
    generated::{Field, FieldId},
    prelude::*,
    FieldRefId,
};
use schema::{ResolverDefinition, ResolverDefinitionId};
use walker::{Iter, Walk};

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type Plan @indexed(id_size: "u16") @meta(module: "plan") {
///   resolver_definition: ResolverDefinition!
///   fields: [Field!]!
///   required_fields: [FieldRef!]!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct PlanRecord {
    pub resolver_definition_id: ResolverDefinitionId,
    pub field_ids: IdRange<FieldId>,
    pub required_field_ids: IdRange<FieldRefId>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub(crate) struct PlanId(std::num::NonZero<u16>);

#[derive(Clone, Copy)]
pub(crate) struct Plan<'a> {
    pub(in crate::plan) ctx: PlanContext<'a>,
    pub(in crate::plan) id: PlanId,
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
        &self.ctx.operation_plan[self.id]
    }
    pub(crate) fn id(&self) -> PlanId {
        self.id
    }
    pub(crate) fn resolver_definition(&self) -> ResolverDefinition<'a> {
        self.resolver_definition_id.walk(self.ctx.schema)
    }
    pub(crate) fn fields(&self) -> impl Iter<Item = Field<'a>> + 'a {
        self.field_ids.walk(self.ctx)
    }
    pub(crate) fn required_fields(&self) -> impl Iter<Item = Field<'a>> + 'a {
        self.required_field_ids.walk(self.ctx)
    }
}

impl<'a> Walk<PlanContext<'a>> for PlanId {
    type Walker<'w> = Plan<'w> where 'a: 'w ;
    fn walk<'w>(self, ctx: PlanContext<'a>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        Plan { ctx, id: self }
    }
}

impl std::fmt::Debug for Plan<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Plan")
            .field("resolver_definition", &self.resolver_definition())
            .field("fields", &self.fields())
            .field("required_fields", &self.required_fields())
            .finish()
    }
}
