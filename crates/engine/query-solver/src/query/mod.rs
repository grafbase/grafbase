pub(crate) mod dot_graph;

use std::collections::HashMap;

use bitflags::bitflags;
use id_newtypes::IdRange;
use operation::{FieldArgumentId, Location, OperationContext, QueryPosition, ResponseKey};
use petgraph::{Graph, visit::GraphBase};
use schema::{
    CompositeTypeId, EntityDefinitionId, FieldDefinitionId, ResolverDefinitionId, SchemaFieldArgumentId, SchemaFieldId,
};
use walker::Walk;

#[derive(Debug, Clone, Copy)]
pub enum Node {
    Root,
    QueryPartition {
        entity_definition_id: EntityDefinitionId,
        resolver_definition_id: ResolverDefinitionId,
    },
    Field {
        id: QueryFieldId,
        flags: FieldFlags,
    },
}

impl Node {
    pub fn as_query_field(&self) -> Option<QueryFieldId> {
        match self {
            Node::Field { id, .. } => Some(*id),
            _ => None,
        }
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct FieldFlags: u8 {
        /// Extra field that is not part of the operation and should not be returned to the user.
        const EXTRA = 1;
        /// Defines whether a field must be requested from the subgraphs. Operations fields are
        /// obviously indispensable, but fields necessary for @authorized also are for example.
        const INDISPENSABLE = 1 << 1;
        /// Whether the field is a leaf node in the Steiner graph.
        const LEAF_NODE = 1 << 2;
        /// Whether the field is a __typename field.
        const TYPENAME = 1 << 3;
        /// If a field ended up being not reachable from a parent type/subgraph we mark it as
        /// unreachable. It might still be possible for it to be resolved from another path though.
        /// It just means that if we couldn't find any resolver for it, we can safely skip it.
        const UNREACHABLE = 1 << 4;
        const PROVIDABLE = 1 << 5;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, strum::IntoStaticStr)]
pub enum Edge {
    QueryPartition,
    Field,
    Derive,
    RequiredBySubgraph,
    RequiredBySupergraph,
    MutationExecutedAfter,
}

pub mod steps {
    pub(crate) struct SteinerTreeSolution;
    pub struct Solution;
}

pub type SolvedQuery = Query<SolutionGraph, steps::Solution>;

pub type SolutionGraph = Graph<Node, Edge>;

#[derive(id_derives::IndexedFields)]
pub struct Query<G: GraphBase, Step> {
    #[allow(unused)] // Used to carry extra data, doesn't anymore.
    pub(crate) step: Step,
    pub root_node_id: G::NodeId,
    pub graph: G,
    #[indexed_by(QueryFieldId)]
    pub fields: Vec<QueryField>,
    #[indexed_by(TypeConditionSharedVecId)]
    pub shared_type_conditions: Vec<CompositeTypeId>,
    pub deduplicated_flat_sorted_executable_directives:
        HashMap<Vec<operation::ExecutableDirectiveId>, DeduplicatedFlatExecutableDirectivesId>,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, id_derives::Id)]
pub struct QueryFieldId(u32);

#[derive(Clone, Copy, id_derives::Id, serde::Serialize, serde::Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct TypeConditionSharedVecId(u32);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, id_derives::Id)]
pub struct DeduplicatedFlatExecutableDirectivesId(std::num::NonZero<u32>);

#[derive(Clone)]
pub struct QueryField {
    pub type_conditions: IdRange<TypeConditionSharedVecId>,
    pub query_position: Option<QueryPosition>,
    pub response_key: Option<ResponseKey>,
    pub subgraph_key: Option<ResponseKey>,
    // If absent it's a typename field.
    pub definition_id: Option<FieldDefinitionId>,
    pub matching_field_id: Option<SchemaFieldId>,
    pub argument_ids: QueryOrSchemaFieldArgumentIds,
    pub location: Location,
    pub flat_directive_id: Option<DeduplicatedFlatExecutableDirectivesId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
pub enum QueryOrSchemaFieldArgumentIds {
    Query(IdRange<FieldArgumentId>),
    Schema(IdRange<SchemaFieldArgumentId>),
}

impl QueryOrSchemaFieldArgumentIds {
    pub fn is_empty(&self) -> bool {
        match self {
            QueryOrSchemaFieldArgumentIds::Query(ids) => ids.is_empty(),
            QueryOrSchemaFieldArgumentIds::Schema(ids) => ids.is_empty(),
        }
    }
}

impl Default for QueryOrSchemaFieldArgumentIds {
    fn default() -> Self {
        QueryOrSchemaFieldArgumentIds::Query(IdRange::empty())
    }
}

impl From<IdRange<FieldArgumentId>> for QueryOrSchemaFieldArgumentIds {
    fn from(ids: IdRange<FieldArgumentId>) -> Self {
        QueryOrSchemaFieldArgumentIds::Query(ids)
    }
}

impl From<IdRange<SchemaFieldArgumentId>> for QueryOrSchemaFieldArgumentIds {
    fn from(ids: IdRange<SchemaFieldArgumentId>) -> Self {
        QueryOrSchemaFieldArgumentIds::Schema(ids)
    }
}

pub(crate) fn are_arguments_equivalent(
    ctx: OperationContext<'_>,
    left: QueryOrSchemaFieldArgumentIds,
    right: QueryOrSchemaFieldArgumentIds,
) -> bool {
    match (left, right) {
        (QueryOrSchemaFieldArgumentIds::Query(left), QueryOrSchemaFieldArgumentIds::Query(right)) => {
            if left.len() != right.len() {
                return false;
            }
            let mut left = left.walk(ctx);
            let mut right = right.walk(ctx);
            let input_values = &ctx.operation.query_input_values;
            while let Some((left_arg, right_arg)) = left.next().zip(right.next()) {
                if left_arg.definition_id != right_arg.definition_id
                    || !operation::are_query_value_equivalent(
                        ctx,
                        &input_values[left_arg.value_id],
                        &input_values[right_arg.value_id],
                    )
                {
                    return false;
                }
            }

            true
        }
        (QueryOrSchemaFieldArgumentIds::Schema(left), QueryOrSchemaFieldArgumentIds::Schema(right)) => {
            if left.len() != right.len() {
                return false;
            }
            let mut left = left.walk(ctx);
            let mut right = right.walk(ctx);
            let schema = ctx.schema;
            while let Some((left_arg, right_arg)) = left.next().zip(right.next()) {
                if left_arg.definition_id != right_arg.definition_id
                    || !left_arg.value_id.walk(schema).eq(&right_arg.value_id.walk(schema))
                {
                    return false;
                }
            }
            true
        }
        (QueryOrSchemaFieldArgumentIds::Query(left), QueryOrSchemaFieldArgumentIds::Schema(right))
        | (QueryOrSchemaFieldArgumentIds::Schema(right), QueryOrSchemaFieldArgumentIds::Query(left)) => {
            if left.len() != right.len() {
                return false;
            }
            let mut left = left.walk(ctx);
            let mut right = right.walk(ctx);
            let input_values = &ctx.operation.query_input_values;
            while let Some((left_arg, right_arg)) = left.next().zip(right.next()) {
                if left_arg.definition_id != right_arg.definition_id
                    || !operation::is_query_value_equivalent_schema_value(
                        ctx,
                        &input_values[left_arg.value_id],
                        &ctx.schema[right_arg.value_id],
                    )
                {
                    return false;
                }
            }

            true
        }
    }
}

impl std::fmt::Debug for SolvedQuery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SolvedQuery").finish_non_exhaustive()
    }
}
