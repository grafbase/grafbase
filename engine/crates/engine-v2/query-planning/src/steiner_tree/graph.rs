use petgraph::{
    data::DataMap,
    graph::{EdgeIndex, Graph, NodeIndex},
    visit::{
        EdgeCount, EdgeIndexable, EdgeRef, GraphBase, IntoEdgeReferences, IntoNodeReferences, NodeCount, NodeIndexable,
    },
};

use crate::Cost;

/// The Steiner Tree algorithm doesn't work directly on the operation graph. We only keep resolver
/// related nodes/edges and build a DAG from it. We call the latter the "Steiner graph" which doesn't
/// have any meaning in the literature.
///
/// The Steiner graph is agnostic of the actual implementation of the operation graph. We
/// create a new one adapted to the algorithm's needs and keep a mapping between the two.
pub(crate) struct SteinerGraph<G: GraphBase> {
    pub(super) operation_graph: G,
    pub(super) graph: Graph<(), Cost>,
    // Mapping between the operation graph and the steiner graph.
    node_ix_to_operation_graph_node_id: Vec<G::NodeId>,
    operation_graph_node_id_to_node_ix: Vec<NodeIndex>,
    operation_graph_edge_id_to_edge_ix: Vec<EdgeIndex>,
}

impl<G: GraphBase> SteinerGraph<G> {
    pub(crate) fn build(
        operation_graph: G,
        node_filter: impl Fn(G::NodeRef) -> Option<G::NodeId>,
        edge_filter: impl Fn(G::EdgeRef) -> Option<(G::EdgeId, G::NodeId, G::NodeId, Cost)>,
    ) -> Self
    where
        G: NodeIndexable + IntoNodeReferences + IntoEdgeReferences + EdgeCount + NodeCount + EdgeIndexable,
        G::EdgeId: Ord,
    {
        let mut graph = Graph::with_capacity(operation_graph.node_count() / 2, operation_graph.edge_count() / 2);

        let mut node_ix_to_operation_graph_node_id = Vec::new();
        let mut operation_graph_node_id_to_node_ix =
            vec![NodeIndex::new(u32::MAX as usize); operation_graph.node_bound()];
        for node_id in operation_graph.node_references().filter_map(node_filter) {
            operation_graph_node_id_to_node_ix[NodeIndexable::to_index(&operation_graph, node_id)] = graph.add_node(());
            node_ix_to_operation_graph_node_id.push(node_id);
        }

        let mut operation_graph_edge_id_to_edge_ix =
            vec![EdgeIndex::new(u32::MAX as usize); operation_graph.edge_bound()];
        for (id, source, target, cost) in operation_graph.edge_references().filter_map(edge_filter) {
            let source_ix = operation_graph_node_id_to_node_ix[NodeIndexable::to_index(&operation_graph, source)];
            let target_ix = operation_graph_node_id_to_node_ix[NodeIndexable::to_index(&operation_graph, target)];
            if source_ix.index() as u32 == u32::MAX || target_ix.index() as u32 == u32::MAX {
                continue;
            }

            let edge_ix = graph.add_edge(source_ix, target_ix, cost);
            operation_graph_edge_id_to_edge_ix[EdgeIndexable::to_index(&operation_graph, id)] = edge_ix;
        }

        Self {
            operation_graph,
            graph,
            node_ix_to_operation_graph_node_id,
            operation_graph_node_id_to_node_ix,
            operation_graph_edge_id_to_edge_ix,
        }
    }

    pub(super) fn to_edge_ix(&self, edge_id: G::EdgeId) -> EdgeIndex
    where
        G: EdgeIndexable + IntoEdgeReferences + DataMap,
        G::EdgeWeight: std::fmt::Debug,
        G::NodeWeight: std::fmt::Debug,
    {
        let ix = self.operation_graph_edge_id_to_edge_ix[self.operation_graph.to_index(edge_id)];
        debug_assert!(ix.index() as u32 != u32::MAX, "{}", {
            let edge_ref = self
                .operation_graph
                .edge_references()
                .find(|edge| edge.id() == edge_id)
                .unwrap();
            format!(
                "{:?}",
                (
                    self.operation_graph.node_weight(edge_ref.source()),
                    self.operation_graph.node_weight(edge_ref.target()),
                    edge_ref.weight(),
                )
            )
        });
        ix
    }

    pub(super) fn to_node_ix(&self, node_id: G::NodeId) -> NodeIndex
    where
        G: NodeIndexable + DataMap,
        G::EdgeId: Ord,
        G::NodeWeight: std::fmt::Debug,
    {
        let ix = self.operation_graph_node_id_to_node_ix[self.operation_graph.to_index(node_id)];
        debug_assert!(
            ix.index() as u32 != u32::MAX,
            "{:?}",
            self.operation_graph.node_weight(node_id)
        );
        ix
    }

    pub(super) fn to_operation_graph_node_id(&self, node_ix: NodeIndex) -> Option<G::NodeId> {
        self.node_ix_to_operation_graph_node_id.get(node_ix.index()).copied()
    }
}
