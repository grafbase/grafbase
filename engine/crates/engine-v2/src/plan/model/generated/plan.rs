//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-v2-codegen`
//! Source file: <engine-v2-codegen dir>/domain/operation_plan.graphql
use crate::plan::model::{
    generated::{
        DataPlanField, PlanSelectionSet, PlanSelectionSetRecord, ResponseObjectSetDefinition,
        ResponseObjectSetDefinitionId,
    },
    prelude::*,
    DataPlanFieldRefId,
};
use schema::{EntityDefinition, EntityDefinitionId, ResolverDefinition, ResolverDefinitionId};
use walker::{Iter, Walk};

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type Plan @indexed(id_size: "u16") @meta(module: "plan") {
///   entity_definition: EntityDefinition!
///   resolver_definition: ResolverDefinition!
///   selection_set: PlanSelectionSet!
///   required_scalar_fields: [DataPlanFieldRef!]!
///   input: ResponseObjectSetDefinition!
///   shape_id: ConcreteObjectShapeId!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct PlanRecord {
    pub entity_definition_id: EntityDefinitionId,
    pub resolver_definition_id: ResolverDefinitionId,
    pub selection_set_record: PlanSelectionSetRecord,
    pub required_scalar_field_ids: IdRange<DataPlanFieldRefId>,
    pub input_id: ResponseObjectSetDefinitionId,
    pub shape_id: ConcreteObjectShapeId,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub(crate) struct PlanId(std::num::NonZero<u16>);

#[derive(Clone, Copy)]
pub(crate) struct Plan<'a> {
    pub(in crate::plan::model) ctx: PlanContext<'a>,
    pub(in crate::plan::model) id: PlanId,
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
    pub(crate) fn entity_definition(&self) -> EntityDefinition<'a> {
        self.entity_definition_id.walk(self.ctx.schema)
    }
    pub(crate) fn resolver_definition(&self) -> ResolverDefinition<'a> {
        self.resolver_definition_id.walk(self.ctx.schema)
    }
    pub(crate) fn selection_set(&self) -> PlanSelectionSet<'a> {
        self.selection_set_record.walk(self.ctx)
    }
    pub(crate) fn required_scalar_fields(&self) -> impl Iter<Item = DataPlanField<'a>> + 'a {
        self.required_scalar_field_ids.walk(self.ctx)
    }
    pub(crate) fn input(&self) -> ResponseObjectSetDefinition<'a> {
        self.input_id.walk(self.ctx)
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
            .field("entity_definition", &self.entity_definition())
            .field("resolver_definition", &self.resolver_definition())
            .field("selection_set", &self.selection_set())
            .field("required_scalar_fields", &self.required_scalar_fields())
            .field("input", &self.input())
            .field("shape_id", &self.shape_id)
            .finish()
    }
}
