//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-codegen`
//! Source file: <engine-codegen dir>/domain/solved_operation.graphql
use crate::operation::solve::model::{
    generated::{
        FieldArgument, FieldArgumentId, QueryPartition, QueryPartitionId, ResponseObjectSetDefinition,
        ResponseObjectSetDefinitionId, SelectionSet, SelectionSetRecord,
    },
    prelude::*,
    DataFieldRefId, FieldShapeRefId,
};
use schema::{FieldDefinition, FieldDefinitionId, SchemaField, SchemaFieldId};
use walker::{Iter, Walk};

/// In opposition to a __typename field this field does retrieve data from a subgraph
///
/// --------------
/// Generated from:
///
/// ```custom,{.language-graphql}
/// type DataField @meta(module: "field/data", debug: false) @indexed(id_size: "u32") {
///   key: PositionedResponseKey!
///   location: Location!
///   definition: FieldDefinition!
///   arguments: [FieldArgument!]!
///   required_fields: [DataFieldRef!]!
///   "Requirement of @authorized, etc."
///   required_fields_by_supergraph: [DataFieldRef!]! @field(record_field_name: "required_field_ids_by_supergraph")
///   "All field shape ids generated for this field"
///   shape_ids: [FieldShapeRefId!]!
///   parent_field_output: ResponseObjectSetDefinition
///   output: ResponseObjectSetDefinition
///   selection_set: SelectionSet!
///   "Whether __typename should be requested from the subgraph for this selection set"
///   selection_set_requires_typename: Boolean!
///   matching_requirement: SchemaField
///   query_partition: QueryPartition!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct DataFieldRecord {
    pub key: PositionedResponseKey,
    pub location: Location,
    pub definition_id: FieldDefinitionId,
    pub argument_ids: IdRange<FieldArgumentId>,
    pub required_field_ids: IdRange<DataFieldRefId>,
    /// Requirement of @authorized, etc.
    pub required_field_ids_by_supergraph: IdRange<DataFieldRefId>,
    /// All field shape ids generated for this field
    pub shape_ids: IdRange<FieldShapeRefId>,
    pub parent_field_output_id: Option<ResponseObjectSetDefinitionId>,
    pub output_id: Option<ResponseObjectSetDefinitionId>,
    pub selection_set_record: SelectionSetRecord,
    /// Whether __typename should be requested from the subgraph for this selection set
    pub selection_set_requires_typename: bool,
    pub matching_requirement_id: Option<SchemaFieldId>,
    pub query_partition_id: QueryPartitionId,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub(crate) struct DataFieldId(std::num::NonZero<u32>);

/// In opposition to a __typename field this field does retrieve data from a subgraph
#[derive(Clone, Copy)]
pub(crate) struct DataField<'a> {
    pub(in crate::operation::solve::model) ctx: SolvedOperationContext<'a>,
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
        &self.ctx.operation[self.id]
    }
    pub(crate) fn definition(&self) -> FieldDefinition<'a> {
        self.definition_id.walk(self.ctx)
    }
    pub(crate) fn arguments(&self) -> impl Iter<Item = FieldArgument<'a>> + 'a {
        self.argument_ids.walk(self.ctx)
    }
    pub(crate) fn required_fields(&self) -> impl Iter<Item = DataField<'a>> + 'a {
        self.required_field_ids.walk(self.ctx)
    }
    /// Requirement of @authorized, etc.
    pub(crate) fn required_fields_by_supergraph(&self) -> impl Iter<Item = DataField<'a>> + 'a {
        self.required_field_ids_by_supergraph.walk(self.ctx)
    }
    pub(crate) fn parent_field_output(&self) -> Option<ResponseObjectSetDefinition<'a>> {
        self.parent_field_output_id.walk(self.ctx)
    }
    pub(crate) fn output(&self) -> Option<ResponseObjectSetDefinition<'a>> {
        self.output_id.walk(self.ctx)
    }
    pub(crate) fn selection_set(&self) -> SelectionSet<'a> {
        self.selection_set_record.walk(self.ctx)
    }
    pub(crate) fn matching_requirement(&self) -> Option<SchemaField<'a>> {
        self.matching_requirement_id.walk(self.ctx)
    }
    pub(crate) fn query_partition(&self) -> QueryPartition<'a> {
        self.query_partition_id.walk(self.ctx)
    }
}

impl<'a> Walk<SolvedOperationContext<'a>> for DataFieldId {
    type Walker<'w> = DataField<'w> where 'a: 'w ;
    fn walk<'w>(self, ctx: impl Into<SolvedOperationContext<'a>>) -> Self::Walker<'w>
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
