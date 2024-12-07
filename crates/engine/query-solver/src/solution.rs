mod dot_graph;

use bitflags::bitflags;
use petgraph::{graph::NodeIndex, Graph};
use schema::{EntityDefinitionId, ResolverDefinitionId, Schema};

use crate::Operation;

#[derive(Debug, Clone, Copy)]
pub enum SolutionNode<FieldId> {
    Root,
    QueryPartition {
        entity_definition_id: EntityDefinitionId,
        resolver_definition_id: ResolverDefinitionId,
    },
    Field {
        id: FieldId,
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
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, strum::IntoStaticStr)]
pub enum SolutionEdge {
    QueryPartition,
    Field,
    RequiredBySubgraph,
    RequiredBySupergraph,
    MutationExecutedAfter,
}

pub struct Solution<'ctx, Op: Operation> {
    pub(crate) schema: &'ctx Schema,
    pub(crate) operation: Op,
    pub root_node_ix: NodeIndex,
    pub graph: SolutionGraph<Op::FieldId>,
}

pub type SolutionGraph<FieldId> = Graph<SolutionNode<FieldId>, SolutionEdge>;

impl<Op: Operation> std::fmt::Debug for Solution<'_, Op> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SolutionGraph").finish_non_exhaustive()
    }
}
