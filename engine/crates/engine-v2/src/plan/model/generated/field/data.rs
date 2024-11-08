//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-v2-codegen`
//! Source file: <engine-v2-codegen dir>/domain/operation_plan.graphql
use crate::plan::model::{
    generated::{
        FieldArgument, FieldArgumentId, Plan, PlanId, PlanSelectionSet, PlanSelectionSetRecord,
        ResponseObjectSetDefinition, ResponseObjectSetDefinitionId,
    },
    prelude::*,
    DataPlanFieldRefId, FieldShapeRefId,
};
use schema::{FieldDefinition, FieldDefinitionId, RequiredField, RequiredFieldId};
use walker::{Iter, Walk};

/// In opposition to a __typename field this field does retrieve data from a subgraph
///
/// --------------
/// Generated from:
///
/// ```custom,{.language-graphql}
/// type DataPlanField @meta(module: "field/data", debug: false) @indexed(id_size: "u32") {
///   key: PositionedResponseKey!
///   location: Location!
///   definition: FieldDefinition!
///   arguments: [FieldArgument!]!
///   "Fields required either by @requires, @authorized, etc."
///   required_scalar_fields: [DataPlanFieldRef!]!
///   "All field shape ids generated for this field"
///   shape_ids: [FieldShapeRefId!]!
///   output: ResponseObjectSetDefinition
///   selection_set: PlanSelectionSet!
///   "Whether __typename should be requested from the subgraph for this selection set"
///   selection_set_requires_typename: Boolean!
///   matching_requirement: RequiredField
///   plan: Plan!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct DataPlanFieldRecord {
    pub key: PositionedResponseKey,
    pub location: Location,
    pub definition_id: FieldDefinitionId,
    pub argument_ids: IdRange<FieldArgumentId>,
    /// Fields required either by @requires, @authorized, etc.
    pub required_scalar_field_ids: IdRange<DataPlanFieldRefId>,
    /// All field shape ids generated for this field
    pub shape_ids: IdRange<FieldShapeRefId>,
    pub output_id: Option<ResponseObjectSetDefinitionId>,
    pub selection_set_record: PlanSelectionSetRecord,
    /// Whether __typename should be requested from the subgraph for this selection set
    pub selection_set_requires_typename: bool,
    pub matching_requirement_id: Option<RequiredFieldId>,
    pub plan_id: PlanId,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub(crate) struct DataPlanFieldId(std::num::NonZero<u32>);

/// In opposition to a __typename field this field does retrieve data from a subgraph
#[derive(Clone, Copy)]
pub(crate) struct DataPlanField<'a> {
    pub(in crate::plan::model) ctx: PlanContext<'a>,
    pub(in crate::plan::model) id: DataPlanFieldId,
}

impl std::ops::Deref for DataPlanField<'_> {
    type Target = DataPlanFieldRecord;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

#[allow(unused)]
impl<'a> DataPlanField<'a> {
    /// Prefer using Deref unless you need the 'a lifetime.
    #[allow(clippy::should_implement_trait)]
    pub(crate) fn as_ref(&self) -> &'a DataPlanFieldRecord {
        &self.ctx.operation_plan[self.id]
    }
    pub(crate) fn id(&self) -> DataPlanFieldId {
        self.id
    }
    pub(crate) fn definition(&self) -> FieldDefinition<'a> {
        self.definition_id.walk(self.ctx.schema)
    }
    pub(crate) fn arguments(&self) -> impl Iter<Item = FieldArgument<'a>> + 'a {
        self.argument_ids.walk(self.ctx)
    }
    /// Fields required either by @requires, @authorized, etc.
    pub(crate) fn required_scalar_fields(&self) -> impl Iter<Item = DataPlanField<'a>> + 'a {
        self.required_scalar_field_ids.walk(self.ctx)
    }
    pub(crate) fn output(&self) -> Option<ResponseObjectSetDefinition<'a>> {
        self.output_id.walk(self.ctx)
    }
    pub(crate) fn selection_set(&self) -> PlanSelectionSet<'a> {
        self.selection_set_record.walk(self.ctx)
    }
    pub(crate) fn matching_requirement(&self) -> Option<RequiredField<'a>> {
        self.matching_requirement_id.walk(self.ctx.schema)
    }
    pub(crate) fn plan(&self) -> Plan<'a> {
        self.plan_id.walk(self.ctx)
    }
}

impl<'a> Walk<PlanContext<'a>> for DataPlanFieldId {
    type Walker<'w> = DataPlanField<'w> where 'a: 'w ;
    fn walk<'w>(self, ctx: PlanContext<'a>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        DataPlanField { ctx, id: self }
    }
}
