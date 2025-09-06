pub(crate) mod dot_graph;

use std::{collections::HashMap, hash::Hash as _, num::NonZero};

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
    Field(FieldNode),
}

#[derive(Debug, Clone, Copy)]
pub struct FieldNode {
    pub id: QueryFieldId,
    pub split_id: SplitId,
    pub flags: FieldFlags,
}

impl FieldNode {
    pub fn is_indispensable(&self) -> bool {
        self.flags.contains(FieldFlags::INDISPENSABLE)
    }

    pub fn is_extra(&self) -> bool {
        self.flags.contains(FieldFlags::EXTRA)
    }

    pub fn is_leaf(&self) -> bool {
        self.flags.contains(FieldFlags::LEAF_NODE)
    }
}

impl Node {
    pub fn as_query_field(&self) -> Option<QueryFieldId> {
        match self {
            Node::Field(node) => Some(node.id),
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

pub(crate) mod steps {

    pub(crate) struct SteinerSolution {}

    pub struct Solution {
        // If necessary we generate a new subgraph key for a field.
        pub field_to_subgraph_key: Vec<Option<operation::ResponseKey>>,
    }
}

pub type QuerySolution = Query<SolutionGraph, steps::Solution>;
pub type SolutionGraph = Graph<Node, Edge>;

#[derive(id_derives::IndexedFields)]
pub struct Query<G: GraphBase, Step> {
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

impl<G: GraphBase, S> std::ops::Deref for Query<G, S> {
    type Target = S;
    fn deref(&self) -> &Self::Target {
        &self.step
    }
}

impl<G: GraphBase, S> std::ops::DerefMut for Query<G, S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.step
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, id_derives::Id)]
pub struct QueryFieldId(u32);

/// Whenever we need to plan an interface through its implementors, we copy query field nodes
/// Later on we rely on those QueryFieldId to de-duplicate ResponseObjectSets between plans,
/// especially in case shared roots. However, contrary to shared roots, here those copied
/// QueryField do represent different objects and today that leads to duplicate plan executions.
#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, id_derives::Id)]
pub struct SplitId(u32);

#[derive(Clone, Copy, id_derives::Id, serde::Serialize, serde::Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeConditionSharedVecId(u32);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, id_derives::Id)]
pub struct DeduplicatedFlatExecutableDirectivesId(NonZero<u32>);

#[derive(Clone)]
pub struct QueryField {
    pub type_conditions: IdRange<TypeConditionSharedVecId>,
    pub flat_directive_id: Option<DeduplicatedFlatExecutableDirectivesId>,
    pub response_key: Option<ResponseKey>,
    // If absent it's a typename field.
    pub definition_id: Option<FieldDefinitionId>,
    pub sorted_argument_ids: QueryOrSchemaSortedFieldArgumentIds,

    // Not relevant for de-duplication.
    pub matching_field_id: Option<SchemaFieldId>,
    pub location: Location,
    pub query_position: Option<QueryPosition>,
}

impl QueryField {
    pub fn equivalence_hash<G: GraphBase, S>(
        &self,
        query: &Query<G, S>,
        ctx: OperationContext<'_>,
        hasher: &mut impl std::hash::Hasher,
    ) {
        query[self.type_conditions].hash(hasher);
        self.flat_directive_id.hash(hasher);
        self.response_key.hash(hasher);
        self.definition_id.hash(hasher);
        self.sorted_argument_ids.len().hash(hasher);
        match self.sorted_argument_ids {
            QueryOrSchemaSortedFieldArgumentIds::Query(ids) => {
                for arg in ids.walk(ctx) {
                    arg.definition_id.hash(hasher);
                }
            }
            QueryOrSchemaSortedFieldArgumentIds::Schema(ids) => {
                for arg in ids.walk(ctx) {
                    arg.definition_id.hash(hasher);
                }
            }
        }
    }

    pub fn is_equivalent<G: GraphBase, S>(
        &self,
        query: &Query<G, S>,
        ctx: OperationContext<'_>,
        other: &QueryField,
    ) -> bool {
        ((query[self.type_conditions] == query[other.type_conditions])
            & (self.response_key == other.response_key)
            & (self.definition_id == other.definition_id)
            & (self.flat_directive_id == other.flat_directive_id))
            && are_arguments_equivalent(ctx, self.sorted_argument_ids, other.sorted_argument_ids)
    }
}

/// Sorted by input value definition id
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
pub enum QueryOrSchemaSortedFieldArgumentIds {
    Query(IdRange<FieldArgumentId>),
    Schema(IdRange<SchemaFieldArgumentId>),
}

impl QueryOrSchemaSortedFieldArgumentIds {
    pub fn is_empty(&self) -> bool {
        match self {
            QueryOrSchemaSortedFieldArgumentIds::Query(ids) => ids.is_empty(),
            QueryOrSchemaSortedFieldArgumentIds::Schema(ids) => ids.is_empty(),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            QueryOrSchemaSortedFieldArgumentIds::Query(ids) => ids.len(),
            QueryOrSchemaSortedFieldArgumentIds::Schema(ids) => ids.len(),
        }
    }
}

pub(crate) fn are_arguments_equivalent(
    ctx: OperationContext<'_>,
    left: QueryOrSchemaSortedFieldArgumentIds,
    right: QueryOrSchemaSortedFieldArgumentIds,
) -> bool {
    match (left, right) {
        (QueryOrSchemaSortedFieldArgumentIds::Query(left), QueryOrSchemaSortedFieldArgumentIds::Query(right)) => {
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
        (QueryOrSchemaSortedFieldArgumentIds::Schema(left), QueryOrSchemaSortedFieldArgumentIds::Schema(right)) => {
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
        (QueryOrSchemaSortedFieldArgumentIds::Query(left), QueryOrSchemaSortedFieldArgumentIds::Schema(right))
        | (QueryOrSchemaSortedFieldArgumentIds::Schema(right), QueryOrSchemaSortedFieldArgumentIds::Query(left)) => {
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

impl std::fmt::Debug for QuerySolution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SolvedQuery").finish_non_exhaustive()
    }
}
