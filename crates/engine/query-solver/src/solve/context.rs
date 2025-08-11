use petgraph::{
    data::DataMap,
    graph::{EdgeIndex, Graph, GraphIndex, NodeIndex},
    stable_graph::EdgeReference,
    visit::{
        EdgeCount, EdgeIndexable, EdgeRef, GraphBase, IntoEdgeReferences, IntoNodeReferences, NodeCount, NodeIndexable,
    },
};

use crate::{Cost, FieldFlags, QuerySolutionSpace, SolutionSpaceGraph, SpaceEdge, SpaceNode};

/// The Steiner Tree algorithm doesn't work directly on the operation graph. We only keep resolver
/// related nodes/edges and build a DAG from it. We call the latter the "Steiner graph" which doesn't
/// have any meaning in the literature.
///
/// The Steiner graph is agnostic of the actual implementation of the operation graph. We
/// create a new one adapted to the algorithm's needs and keep a mapping between the two.
pub(crate) struct SteinerContext<SpaceGraph: GraphBase, G: GraphBase> {
    pub(crate) space_graph: SpaceGraph,
    pub(crate) graph: G,
    pub(crate) root_ix: G::NodeId,
    // Mapping between the operation graph and the steiner graph.
    node_ix_to_space_graph_node_id: Vec<SpaceGraph::NodeId>,
    pub(crate) space_graph_node_id_to_node_ix: Vec<G::NodeId>,
    space_graph_edge_id_to_edge_ix: Vec<G::EdgeId>,
}

pub(in crate::solve) type SteinerGraph = Graph<(), Cost>;
pub(in crate::solve) type SteinerNodeId = <SteinerGraph as GraphBase>::NodeId;
pub(in crate::solve) type SteinerEdgeId = <SteinerGraph as GraphBase>::EdgeId;

impl<'g, 'schema> SteinerContext<&'g SolutionSpaceGraph<'schema>, SteinerGraph> {
    pub(crate) fn from_query_solution_space(
        query_solution_space: &'g QuerySolutionSpace<'schema>,
    ) -> (Self, Vec<NodeIndex>) {
        let node_filter = |(node_ix, node): (NodeIndex, &SpaceNode<'schema>)| match node {
            SpaceNode::Root | SpaceNode::Resolver(_) | SpaceNode::ProvidableField(_) => Some(node_ix),
            SpaceNode::QueryField(field) => {
                if field.is_leaf() {
                    Some(node_ix)
                } else {
                    None
                }
            }
        };
        let edge_filter = |edge: EdgeReference<'_, SpaceEdge, _>| match edge.weight() {
            // Resolvers have an inherent cost of 1.
            SpaceEdge::CreateChildResolver => Some((edge.id(), edge.source(), edge.target(), 1)),
            SpaceEdge::CanProvide | SpaceEdge::Provides | SpaceEdge::TypenameField => {
                Some((edge.id(), edge.source(), edge.target(), 0))
            }
            SpaceEdge::Field | SpaceEdge::HasChildResolver | SpaceEdge::Requires => None,
        };
        let ctx = Self::build(
            &query_solution_space.graph,
            query_solution_space.root_node_id,
            node_filter,
            edge_filter,
        );

        let mut terminals = Vec::new();
        for (node_ix, node) in query_solution_space.graph.node_references() {
            if let SpaceNode::QueryField(field) = node
                && field.flags.contains(FieldFlags::LEAF_NODE | FieldFlags::INDISPENSABLE)
            {
                terminals.push(ctx.to_node_ix(node_ix));
            }
        }

        (ctx, terminals)
    }
}

impl<QG: GraphBase> SteinerContext<QG, SteinerGraph> {
    pub(crate) fn build(
        space_graph: QG,
        root_ix: QG::NodeId,
        node_filter: impl Fn(QG::NodeRef) -> Option<QG::NodeId>,
        edge_filter: impl Fn(QG::EdgeRef) -> Option<(QG::EdgeId, QG::NodeId, QG::NodeId, Cost)>,
    ) -> Self
    where
        QG: NodeIndexable + IntoNodeReferences + IntoEdgeReferences + EdgeCount + NodeCount + EdgeIndexable,
        QG::EdgeId: GraphIndex + Ord,
        QG::NodeId: GraphIndex,
    {
        let mut graph = Graph::with_capacity(space_graph.node_count() / 2, space_graph.edge_count() / 2);

        let mut node_ix_to_space_graph_node_id = Vec::new();
        let mut space_graph_node_id_to_node_ix = vec![NodeIndex::new(u32::MAX as usize); space_graph.node_bound()];
        for node_id in space_graph.node_references().filter_map(node_filter) {
            space_graph_node_id_to_node_ix[node_id.index()] = graph.add_node(());
            node_ix_to_space_graph_node_id.push(node_id);
        }

        let mut space_graph_edge_id_to_edge_ix = vec![EdgeIndex::new(u32::MAX as usize); space_graph.edge_bound()];
        for (id, source, target, cost) in space_graph.edge_references().filter_map(edge_filter) {
            let source_ix = space_graph_node_id_to_node_ix[source.index()];
            let target_ix = space_graph_node_id_to_node_ix[target.index()];
            if source_ix.index() as u32 == u32::MAX || target_ix.index() as u32 == u32::MAX {
                continue;
            }

            let edge_ix = graph.add_edge(source_ix, target_ix, cost);
            space_graph_edge_id_to_edge_ix[id.index()] = edge_ix;
        }

        let root_ix = space_graph_node_id_to_node_ix[root_ix.index()];
        Self {
            space_graph,
            graph,
            root_ix,
            node_ix_to_space_graph_node_id,
            space_graph_node_id_to_node_ix,
            space_graph_edge_id_to_edge_ix,
        }
    }
}

impl<QG: GraphBase, G: GraphBase> SteinerContext<QG, G> {
    pub(crate) fn to_edge_ix(&self, edge_id: QG::EdgeId) -> G::EdgeId
    where
        G::EdgeId: GraphIndex,
        QG::EdgeId: GraphIndex,
        QG: EdgeIndexable + IntoEdgeReferences + DataMap,
        QG::EdgeWeight: std::fmt::Debug,
        QG::NodeWeight: std::fmt::Debug,
    {
        let ix = self.space_graph_edge_id_to_edge_ix[edge_id.index()];
        debug_assert!(ix.index() as u32 != u32::MAX, "{}", {
            let edge_ref = self
                .space_graph
                .edge_references()
                .find(|edge| edge.id() == edge_id)
                .unwrap();
            format!(
                "{:?}",
                (
                    self.space_graph.node_weight(edge_ref.source()),
                    self.space_graph.node_weight(edge_ref.target()),
                    edge_ref.weight(),
                )
            )
        });
        ix
    }

    pub(crate) fn to_node_ix(&self, node_id: QG::NodeId) -> G::NodeId
    where
        G::NodeId: GraphIndex,
        QG::NodeId: GraphIndex,
        QG: NodeIndexable + DataMap,
        QG::EdgeId: Ord,
        QG::NodeWeight: std::fmt::Debug,
    {
        let ix = self.space_graph_node_id_to_node_ix[node_id.index()];
        debug_assert!(
            ix.index() as u32 != u32::MAX,
            "{:?}",
            self.space_graph.node_weight(node_id)
        );
        ix
    }

    pub(crate) fn to_space_graph_node_id(&self, node_ix: G::NodeId) -> Option<QG::NodeId>
    where
        G::NodeId: GraphIndex,
    {
        self.node_ix_to_space_graph_node_id.get(node_ix.index()).copied()
    }
}
