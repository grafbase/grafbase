pub(crate) mod dot_graph;

use std::marker::PhantomData;

use bitflags::bitflags;
use id_newtypes::IdRange;
use operation::{FieldArgumentId, Location, QueryPosition, ResponseKey};
use petgraph::{visit::GraphBase, Graph};
use schema::{CompositeTypeId, EntityDefinitionId, FieldDefinitionId, ResolverDefinitionId, SchemaFieldArgumentId};

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
        /// Whether the field output is a composite type
        const IS_COMPOSITE_TYPE = 1 << 4;
        /// If a field ended up being not reachable from a parent type/subgraph we mark it as
        /// unreachable. It might still be possible for it to be resolved from another path though.
        /// It just means that if we couldn't find any resolver for it, we can safely skip it.
        const UNREACHABLE = 1 << 5;
        const PROVIDABLE = 1 << 6;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, strum::IntoStaticStr)]
pub enum Edge {
    QueryPartition,
    Field,
    RequiredBySubgraph,
    RequiredBySupergraph,
    MutationExecutedAfter,
}

pub mod steps {
    pub(crate) struct SolutionSpace;
    pub(crate) struct SteinerTreeSolution;
    pub struct Solution;
}

pub type SolvedQuery = Query<SolutionGraph, steps::Solution>;

pub type SolutionGraph = Graph<Node, Edge>;

#[derive(id_derives::IndexedFields)]
pub struct Query<G: GraphBase, Step> {
    pub(crate) step: PhantomData<Step>,
    pub root_ix: G::NodeId,
    pub graph: G,
    #[indexed_by(QueryFieldId)]
    pub fields: Vec<QueryField>,
    #[indexed_by(TypeConditionSharedVecId)]
    pub shared_type_conditions: Vec<CompositeTypeId>,
    #[indexed_by(DirectiveSharedVecId)]
    pub shared_directives: Vec<operation::ExecutableDirectiveId>,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, id_derives::Id)]
pub struct QueryFieldId(u32);

#[derive(Clone, Copy, id_derives::Id)]
pub struct TypeConditionSharedVecId(u32);

#[derive(Clone, Copy, id_derives::Id)]
pub struct DirectiveSharedVecId(u32);

#[derive(Clone)]
pub struct QueryField {
    pub type_conditions: IdRange<TypeConditionSharedVecId>,
    pub query_position: Option<QueryPosition>,
    pub key: Option<ResponseKey>,
    pub subgraph_key: Option<ResponseKey>,
    // If absent it's a typename field.
    pub definition_id: Option<FieldDefinitionId>,
    pub argument_ids: FieldArguments,
    pub location: Location,
    pub directive_ids: IdRange<DirectiveSharedVecId>,
}

#[derive(Clone)]
pub enum FieldArguments {
    Original(IdRange<FieldArgumentId>),
    Extra(IdRange<SchemaFieldArgumentId>),
}

impl Default for FieldArguments {
    fn default() -> Self {
        FieldArguments::Original(IdRange::empty())
    }
}

impl From<IdRange<FieldArgumentId>> for FieldArguments {
    fn from(ids: IdRange<FieldArgumentId>) -> Self {
        FieldArguments::Original(ids)
    }
}

impl From<IdRange<SchemaFieldArgumentId>> for FieldArguments {
    fn from(ids: IdRange<SchemaFieldArgumentId>) -> Self {
        FieldArguments::Extra(ids)
    }
}

impl std::fmt::Debug for SolvedQuery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SolvedQuery").finish_non_exhaustive()
    }
}
