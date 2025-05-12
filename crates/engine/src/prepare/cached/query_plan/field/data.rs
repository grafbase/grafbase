use id_newtypes::IdRange;
use operation::{Location, PositionedResponseKey, QueryPosition, ResponseKey};
use query_solver::TypeConditionSharedVecId;
use schema::{CompositeType, FieldDefinition, FieldDefinitionId};
use walker::{Iter, Walk};

use crate::prepare::{
    CachedOperationContext, DataOrLookupFieldId, FieldShapeId, RequiredFieldSet,
    cached::query_plan::{
        FieldShapeRefId, PartitionSelectionSet, PartitionSelectionSetRecord, QueryPartitionId, RequiredFieldSetRecord,
        ResponseObjectSetDefinitionId,
    },
};

use super::{DataOrLookupField, PartitionFieldArgumentId, PlanFieldArguments};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct DataFieldRecord {
    pub type_condition_ids: IdRange<TypeConditionSharedVecId>,
    pub query_position: Option<QueryPosition>,
    pub response_key: ResponseKey,
    pub subgraph_key: Option<ResponseKey>,
    pub location: Location,
    pub argument_ids: IdRange<PartitionFieldArgumentId>,
    pub definition_id: FieldDefinitionId,
    pub derived: Option<Derived>,

    pub required_fields_record: RequiredFieldSetRecord,
    /// Requirement of @authorized, etc.
    pub required_fields_record_by_supergraph: RequiredFieldSetRecord,
    /// All field shape ids generated for this field
    pub shape_ids_ref: IdRange<FieldShapeRefId>,
    pub parent_field_id: Option<DataOrLookupFieldId>,
    pub output_id: Option<ResponseObjectSetDefinitionId>,
    pub selection_set_record: PartitionSelectionSetRecord,
    /// Whether __typename should be requested from the subgraph for this selection set
    pub selection_set_requires_typename: bool,
    pub query_partition_id: QueryPartitionId,
}

impl DataFieldRecord {
    pub(crate) fn key(&self) -> PositionedResponseKey {
        PositionedResponseKey {
            query_position: self.query_position,
            response_key: self.response_key,
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) enum Derived {
    Root,
    From(DataFieldId),
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub(crate) struct DataFieldId(u16);

/// In opposition to a __typename field this field does retrieve data from a subgraph
#[derive(Clone, Copy)]
pub(crate) struct DataField<'a> {
    pub(in crate::prepare::cached::query_plan) ctx: CachedOperationContext<'a>,
    pub(crate) id: DataFieldId,
}

impl std::ops::Deref for DataField<'_> {
    type Target = DataFieldRecord;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

#[allow(unused)]
impl<'a> DataField<'a> {
    /// Prefer using Deref unless you need the 'a lifetime.
    #[allow(clippy::should_implement_trait)]
    pub(crate) fn as_ref(&self) -> &'a DataFieldRecord {
        &self.ctx.cached.query_plan[self.id]
    }
    pub(crate) fn type_conditions(&self) -> impl Iter<Item = CompositeType<'a>> {
        self.ctx.cached.query_plan[self.as_ref().type_condition_ids].walk(self.ctx)
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
    pub(crate) fn parent_field(&self) -> Option<DataOrLookupField<'a>> {
        self.as_ref().parent_field_id.walk(self.ctx)
    }
    pub(crate) fn shape_ids(&self) -> impl Iter<Item = FieldShapeId> + 'a {
        self.ctx.cached.query_plan[self.as_ref().shape_ids_ref].iter().copied()
    }
}

impl std::fmt::Debug for DataField<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DataField")
            .field("key", &&self.ctx.cached.operation.response_keys[self.response_key])
            .field("definition", &self.definition())
            .field("selection_set", &self.selection_set())
            .finish()
    }
}

impl<'a> Walk<CachedOperationContext<'a>> for DataFieldId {
    type Walker<'w>
        = DataField<'w>
    where
        'a: 'w;
    fn walk<'w>(self, ctx: impl Into<CachedOperationContext<'a>>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        DataField {
            ctx: ctx.into(),
            id: self,
        }
    }
}
