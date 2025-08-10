use petgraph::{
    data::DataMap,
    graph::{EdgeIndex, Graph, GraphIndex, NodeIndex},
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
pub(crate) struct SteinerContext<QueryGraph: GraphBase, G: GraphBase> {
    pub(super) query_graph: QueryGraph,
    pub(super) graph: G,
    pub(super) root_ix: G::NodeId,
    // Mapping between the operation graph and the steiner graph.
    node_ix_to_query_graph_node_id: Vec<QueryGraph::NodeId>,
    pub(super) query_graph_node_id_to_node_ix: Vec<G::NodeId>,
    query_graph_edge_id_to_edge_ix: Vec<G::EdgeId>,
}

pub(in crate::solve) type SteinerGraph = Graph<(), Cost>;

impl<QG: GraphBase> SteinerContext<QG, SteinerGraph> {
    pub(crate) fn build(
        query_graph: QG,
        root_ix: QG::NodeId,
        node_filter: impl Fn(QG::NodeRef) -> Option<QG::NodeId>,
        edge_filter: impl Fn(QG::EdgeRef) -> Option<(QG::EdgeId, QG::NodeId, QG::NodeId, Cost)>,
    ) -> Self
    where
        QG: NodeIndexable + IntoNodeReferences + IntoEdgeReferences + EdgeCount + NodeCount + EdgeIndexable,
        QG::EdgeId: GraphIndex + Ord,
        QG::NodeId: GraphIndex,
    {
        let mut graph = Graph::with_capacity(query_graph.node_count() / 2, query_graph.edge_count() / 2);

        let mut node_ix_to_query_graph_node_id = Vec::new();
        let mut query_graph_node_id_to_node_ix = vec![NodeIndex::new(u32::MAX as usize); query_graph.node_bound()];
        for node_id in query_graph.node_references().filter_map(node_filter) {
            query_graph_node_id_to_node_ix[node_id.index()] = graph.add_node(());
            node_ix_to_query_graph_node_id.push(node_id);
        }

        let mut query_graph_edge_id_to_edge_ix = vec![EdgeIndex::new(u32::MAX as usize); query_graph.edge_bound()];
        for (id, source, target, cost) in query_graph.edge_references().filter_map(edge_filter) {
            let source_ix = query_graph_node_id_to_node_ix[source.index()];
            let target_ix = query_graph_node_id_to_node_ix[target.index()];
            if source_ix.index() as u32 == u32::MAX || target_ix.index() as u32 == u32::MAX {
                continue;
            }

            let edge_ix = graph.add_edge(source_ix, target_ix, cost);
            query_graph_edge_id_to_edge_ix[id.index()] = edge_ix;
        }

        let root_ix = query_graph_node_id_to_node_ix[root_ix.index()];
        Self {
            query_graph,
            graph,
            root_ix,
            node_ix_to_query_graph_node_id,
            query_graph_node_id_to_node_ix,
            query_graph_edge_id_to_edge_ix,
        }
    }
}

impl<QG: GraphBase, G: GraphBase> SteinerContext<QG, G> {
    pub(super) fn to_edge_ix(&self, edge_id: QG::EdgeId) -> G::EdgeId
    where
        G::EdgeId: GraphIndex,
        QG::EdgeId: GraphIndex,
        QG: EdgeIndexable + IntoEdgeReferences + DataMap,
        QG::EdgeWeight: std::fmt::Debug,
        QG::NodeWeight: std::fmt::Debug,
    {
        let ix = self.query_graph_edge_id_to_edge_ix[edge_id.index()];
        debug_assert!(ix.index() as u32 != u32::MAX, "{}", {
            let edge_ref = self
                .query_graph
                .edge_references()
                .find(|edge| edge.id() == edge_id)
                .unwrap();
            format!(
                "{:?}",
                (
                    self.query_graph.node_weight(edge_ref.source()),
                    self.query_graph.node_weight(edge_ref.target()),
                    edge_ref.weight(),
                )
            )
        });
        ix
    }

    pub(super) fn to_node_ix(&self, node_id: QG::NodeId) -> G::NodeId
    where
        G::NodeId: GraphIndex,
        QG::NodeId: GraphIndex,
        QG: NodeIndexable + DataMap,
        QG::EdgeId: Ord,
        QG::NodeWeight: std::fmt::Debug,
    {
        let ix = self.query_graph_node_id_to_node_ix[node_id.index()];
        debug_assert!(
            ix.index() as u32 != u32::MAX,
            "{:?}",
            self.query_graph.node_weight(node_id)
        );
        ix
    }

    pub(super) fn to_query_graph_node_id(&self, node_ix: G::NodeId) -> Option<QG::NodeId>
    where
        G::NodeId: GraphIndex,
    {
        self.node_ix_to_query_graph_node_id.get(node_ix.index()).copied()
    }
}
