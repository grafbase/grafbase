use id_newtypes::IdRange;
use operation::{Location, PositionedResponseKey, QueryPosition, ResponseKey};
use query_solver::TypeConditionSharedVecId;
use schema::{CompositeType, FieldDefinition, FieldDefinitionId};
use walker::{Iter, Walk};

use crate::prepare::{
    CachedOperationContext, RequiredFieldSet,
    cached::query_plan::{
        FieldShapeRefId, PartitionSelectionSet, PartitionSelectionSetRecord, QueryPartitionId, RequiredFieldSetRecord,
        ResponseObjectSetDefinitionId,
    },
};

use super::PartitionFieldArguments;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct PartitionDataFieldRecord {
    pub type_condition_ids: IdRange<TypeConditionSharedVecId>,
    pub query_position: Option<QueryPosition>,
    pub response_key: ResponseKey,
    pub subgraph_key: Option<ResponseKey>,
    pub location: Location,
    pub argument_ids: query_solver::QueryOrSchemaFieldArgumentIds,
    pub definition_id: FieldDefinitionId,

    pub required_fields_record: RequiredFieldSetRecord,
    /// Requirement of @authorized, etc.
    pub required_fields_record_by_supergraph: RequiredFieldSetRecord,
    /// All field shape ids generated for this field
    pub shape_ids: IdRange<FieldShapeRefId>,
    pub parent_field_id: Option<PartitionDataFieldId>,
    pub output_id: Option<ResponseObjectSetDefinitionId>,
    pub selection_set_record: PartitionSelectionSetRecord,
    /// Whether __typename should be requested from the subgraph for this selection set
    pub selection_set_requires_typename: bool,
    pub query_partition_id: QueryPartitionId,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub(crate) struct PartitionDataFieldId(u16);

/// In opposition to a __typename field this field does retrieve data from a subgraph
#[derive(Clone, Copy)]
pub(crate) struct PartitionDataField<'a> {
    pub(in crate::prepare::cached::query_plan) ctx: CachedOperationContext<'a>,
    pub(crate) id: PartitionDataFieldId,
}

impl std::ops::Deref for PartitionDataField<'_> {
    type Target = PartitionDataFieldRecord;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

#[allow(unused)]
impl<'a> PartitionDataField<'a> {
    /// Prefer using Deref unless you need the 'a lifetime.
    #[allow(clippy::should_implement_trait)]
    pub(crate) fn as_ref(&self) -> &'a PartitionDataFieldRecord {
        &self.ctx.cached.query_plan[self.id]
    }
    pub(crate) fn type_conditions(&self) -> impl Iter<Item = CompositeType<'a>> {
        self.ctx.cached.query_plan[self.as_ref().type_condition_ids].walk(self.ctx)
    }
    pub(crate) fn definition(&self) -> FieldDefinition<'a> {
        self.definition_id.walk(self.ctx)
    }
    pub(crate) fn arguments(&self) -> PartitionFieldArguments<'a> {
        self.as_ref().argument_ids.walk(self.ctx)
    }
    pub(crate) fn selection_set(&self) -> PartitionSelectionSet<'a> {
        self.selection_set_record.walk(self.ctx)
    }
    pub(crate) fn required_fields_by_supergraph(&self) -> RequiredFieldSet<'a> {
        self.as_ref().required_fields_record_by_supergraph.walk(self.ctx)
    }
    pub(crate) fn parent_field(&self) -> Option<PartitionDataField<'a>> {
        self.as_ref().parent_field_id.walk(self.ctx)
    }
    pub(crate) fn key(&self) -> PositionedResponseKey {
        PositionedResponseKey {
            query_position: self.query_position,
            response_key: self.response_key,
        }
    }
}

impl<'a> Walk<CachedOperationContext<'a>> for PartitionDataFieldId {
    type Walker<'w>
        = PartitionDataField<'w>
    where
        'a: 'w;
    fn walk<'w>(self, ctx: impl Into<CachedOperationContext<'a>>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        PartitionDataField {
            ctx: ctx.into(),
            id: self,
        }
    }
}
