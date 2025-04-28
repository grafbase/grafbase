use id_newtypes::IdRange;
use operation::{Location, ResponseKey};
use schema::{FieldDefinition, FieldDefinitionId};
use walker::Walk;

use crate::prepare::{
    CachedOperationContext, RequiredFieldSet,
    cached::query_plan::{
        FieldShapeRefId, PartitionSelectionSet, PartitionSelectionSetRecord, QueryPartitionId, RequiredFieldSetRecord,
        ResponseObjectSetDefinitionId,
    },
};

use super::{PartitionFieldArgumentId, PlanFieldArguments};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct LookupFieldRecord {
    pub subgraph_key: ResponseKey,
    pub location: Location,
    pub argument_ids: IdRange<PartitionFieldArgumentId>,
    pub definition_id: FieldDefinitionId,

    /// Requirement of @authorized, etc.
    pub required_fields_record_by_supergraph: RequiredFieldSetRecord,
    /// All field shape ids generated for this field
    pub shape_ids: IdRange<FieldShapeRefId>,
    pub output_id: Option<ResponseObjectSetDefinitionId>,
    pub selection_set_record: PartitionSelectionSetRecord,
    pub query_partition_id: QueryPartitionId,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub(crate) struct LookupFieldId(u16);

/// In opposition to a __typename field this field does retrieve data from a subgraph
#[derive(Clone, Copy)]
pub(crate) struct LookupField<'a> {
    pub(in crate::prepare::cached::query_plan) ctx: CachedOperationContext<'a>,
    pub(crate) id: LookupFieldId,
}

impl std::ops::Deref for LookupField<'_> {
    type Target = LookupFieldRecord;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

#[allow(unused)]
impl<'a> LookupField<'a> {
    /// Prefer using Deref unless you need the 'a lifetime.
    #[allow(clippy::should_implement_trait)]
    pub(crate) fn as_ref(&self) -> &'a LookupFieldRecord {
        &self.ctx.cached.query_plan[self.id]
    }
    pub(crate) fn definition(&self) -> FieldDefinition<'a> {
        self.definition_id.walk(self.ctx)
    }
    pub(crate) fn arguments(&self) -> PlanFieldArguments<'a> {
        PlanFieldArguments {
            ctx: self.ctx,
            records: &self.ctx.cached.query_plan[self.as_ref().argument_ids],
        }
    }
    pub(crate) fn selection_set(&self) -> PartitionSelectionSet<'a> {
        self.selection_set_record.walk(self.ctx)
    }
    pub(crate) fn required_fields_by_supergraph(&self) -> RequiredFieldSet<'a> {
        self.as_ref().required_fields_record_by_supergraph.walk(self.ctx)
    }
}

impl std::fmt::Debug for LookupField<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DataPlanField")
            .field("key", &self.subgraph_key)
            .field("location", &self.location)
            .field("definition", &self.definition())
            .field("selection_set", &self.selection_set())
            .finish()
    }
}

impl<'a> Walk<CachedOperationContext<'a>> for LookupFieldId {
    type Walker<'w>
        = LookupField<'w>
    where
        'a: 'w;
    fn walk<'w>(self, ctx: impl Into<CachedOperationContext<'a>>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        LookupField {
            ctx: ctx.into(),
            id: self,
        }
    }
}
