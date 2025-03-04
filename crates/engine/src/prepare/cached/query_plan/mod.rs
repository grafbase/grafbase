mod field;
mod generated;
mod modifier;
mod prelude;
mod required_field_set;
mod selection_set;

use extension_catalog::ExtensionId;
pub(crate) use field::*;
pub(crate) use generated::*;
use id_newtypes::{BitSet, IdRange};
pub(crate) use modifier::*;
use query_solver::TypeConditionSharedVecId;
pub(crate) use required_field_set::*;
use schema::{CompositeTypeId, StringId};

use super::FieldShapeId;

/// The solved operation contains almost all the necessary data to execute the operation. It only
/// needs to be adjusted with `@skip`, `@include` etc.. This latter step produces the
/// OperationPlan. If the operation doesn't involve any skip, include or authorized directive it's
/// effectively all the information there is to know.
///
/// The solved operation is fundamentally a partitioning of the original query into QueryPartitions,
/// each associated with a ResolverDefinition and field/object shapes. The latter define the
/// structure we expect to retrieve from the subgraph response.
///
/// Only query partitions resolving root fields in a mutation are ordered. Otherwise there is no
/// direct relationship between them. Dependencies are tracked at the field level:
/// - ResolverDefinition requirements -> `QueryPartition.required_fields`
/// - `@requires` -> `DataField.required_fields`
/// - `@authorized` requirements -> `DataField.required_fields_by_supergraph`
///
/// When building the OperationPlan, taking into account skip, include and unauthorized fields, we
/// infer the ordering of the plans and response modifiers from those requirements. This allows us
/// to run as efficiently as possible the different steps of the plan, only waiting for relevant
/// data.
#[derive(id_derives::IndexedFields, serde::Serialize, serde::Deserialize)]
pub(crate) struct QueryPlan {
    #[indexed_by(QueryPartitionId)]
    pub partitions: Vec<QueryPartitionRecord>,
    #[indexed_by(PartitionDataFieldId)]
    pub data_fields: Vec<PartitionDataFieldRecord>,
    pub response_data_fields: BitSet<PartitionDataFieldId>,
    #[indexed_by(PartitionTypenameFieldId)]
    pub typename_fields: Vec<PartitionTypenameFieldRecord>,
    pub response_typename_fields: BitSet<PartitionTypenameFieldId>,
    pub mutation_partition_order: Vec<QueryPartitionId>,
    #[indexed_by(TypeConditionSharedVecId)]
    pub shared_type_conditions: Vec<CompositeTypeId>,

    pub query_modifiers: QueryModifiers,
    pub response_modifier_definitions: Vec<ResponseModifierDefinitionRecord>,

    pub root_response_object_set_id: ResponseObjectSetDefinitionId,
    #[indexed_by(ResponseObjectSetDefinitionId)]
    pub response_object_set_definitions: Vec<ResponseObjectSetDefinitionRecord>,

    // Refs are used to replace a Vec<XId> with a IdRange<XRefId>. IdRange<XRefId> will at most have a size
    // of 2 * u32 while Vec<XId> is 3 words long. And we store everything in a single Vec.
    #[indexed_by(FieldShapeRefId)]
    pub field_shape_refs: Vec<FieldShapeId>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub(crate) struct FieldShapeRefId(u32);

#[derive(Default, id_derives::IndexedFields, serde::Serialize, serde::Deserialize)]
pub(crate) struct QueryModifiers {
    pub native_ids: IdRange<QueryModifierId>,
    pub by_extension: Vec<(
        ExtensionId,
        IdRange<QueryModifierByDirectiveGroupId>,
        IdRange<QueryModifierId>,
    )>,
    #[indexed_by(QueryModifierByDirectiveGroupId)]
    pub by_directive: Vec<(StringId, IdRange<QueryModifierId>)>,
    // deduplicated by rule
    // sorted by ExtensionId, directive name
    #[indexed_by(QueryModifierId)]
    pub records: Vec<QueryModifierRecord>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub(crate) struct QueryModifierByDirectiveGroupId(u32);

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub(crate) struct QueryModifierId(u32);
