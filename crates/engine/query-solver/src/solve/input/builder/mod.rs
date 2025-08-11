mod requirements;

use petgraph::{
    Direction,
    graph::{Graph, NodeIndex},
    visit::{EdgeRef, NodeIndexable},
};

use crate::{
    Cost, QuerySolutionSpace, SpaceEdge, SpaceEdgeId, SpaceNodeId,
    solve::{
        context::{SteinerGraph, SteinerNodeId},
        input::builder::requirements::DispensableRequirementsBuilder,
    },
};
pub(crate) struct SteinerInputBuilder {
    pub graph: SteinerGraph,
    pub root_node_id: SteinerNodeId,
    pub node_id_to_space_node_id: Vec<SpaceNodeId>,
    pub edge_id_to_space_edge_id: Vec<SpaceEdgeId>,
    pub space_node_id_to_node_id: Vec<NodeIndex>,
    pub terminals_to_process: Vec<SpaceNodeId>,
    pub nodes_to_process: Vec<NodeToProcess>,
}

struct NodeToProcess {
    space_node_id: SpaceNodeId,
    // u32::MAX if doesn't exist.
    maybe_child_node_id: SteinerNodeId,
    // Only valid if maybe_child_node_id is not u32::MAX.
    maybe_child_space_edge_id: SpaceEdgeId,
    maybe_child_edge_cost: Cost,
}

impl NodeToProcess {
    pub fn terminal(space_node_id: SpaceNodeId) -> Self {
        Self {
            space_node_id,
            maybe_child_node_id: SteinerNodeId::new(u32::MAX as usize),
            maybe_child_space_edge_id: SpaceEdgeId::new(u32::MAX as usize),
            maybe_child_edge_cost: 0,
        }
    }
}

impl SteinerInputBuilder {
    pub fn build<'schema>(mut space: QuerySolutionSpace<'schema>) -> super::SteinerInput<'schema> {
        let mut graph = Graph::with_capacity(space.graph.node_bound() >> 3, space.graph.edge_count() >> 3);
        let mut node_id_to_space_node_id = Vec::with_capacity(graph.node_bound());
        let edge_id_to_space_edge_id = Vec::with_capacity(graph.edge_count());
        let mut space_node_id_to_node_id = vec![NodeIndex::new(u32::MAX as usize); space.graph.node_bound()];

        node_id_to_space_node_id.push(space.root_node_ix);
        let root_node_id = graph.add_node(());
        space_node_id_to_node_id[space.root_node_ix.index()] = root_node_id;
        let mut builder = Self {
            terminals_to_process: std::mem::take(&mut space.step.indispensable_leaf_nodes),
            graph,
            root_node_id,
            node_id_to_space_node_id,
            edge_id_to_space_edge_id,
            space_node_id_to_node_id,
            nodes_to_process: Vec::new(),
        };
        let mut requirements = DispensableRequirementsBuilder::new(&space.graph);

        while let Some(space_terminal) = builder.terminals_to_process.pop() {
            debug_assert!(builder.nodes_to_process.is_empty());
            builder.nodes_to_process.push(NodeToProcess::terminal(space_terminal));
            builder.ingest_nodes_from_indispensable_terminal(&space, &mut requirements);
        }

        super::SteinerInput {
            space,
            graph: builder.graph,
            node_id_to_space_node_id: builder.node_id_to_space_node_id,
            edge_id_to_space_edge_id: builder.edge_id_to_space_edge_id,
            space_node_id_to_node_id: builder.space_node_id_to_node_id,
            requirements: requirements.build(),
        }
    }

    pub fn ingest_nodes_from_indispensable_terminal(
        &mut self,
        space: &QuerySolutionSpace<'_>,
        requirements: &mut DispensableRequirementsBuilder,
    ) {
        let mut indispensable = true;

        while let Some(NodeToProcess {
            space_node_id,
            maybe_child_node_id,
            maybe_child_space_edge_id,
            maybe_child_edge_cost,
        }) = self.nodes_to_process.pop()
        {
            let maybe_existing = self.space_node_id_to_node_id[space_node_id.index()];
            if maybe_existing.index() != u32::MAX as usize {
                // Already processed, just add the edge.
                if maybe_child_node_id.index() != u32::MAX as usize {
                    self.graph
                        .add_edge(maybe_existing, maybe_child_node_id, maybe_child_edge_cost);
                    self.edge_id_to_space_edge_id.push(maybe_child_space_edge_id);
                }
                continue;
            }

            let mut space_edges = space
                .graph
                .edges_directed(space_node_id, Direction::Incoming)
                .filter_map(|space_edge| match space_edge.weight() {
                    SpaceEdge::CreateChildResolver => Some((space_edge, 1)),
                    SpaceEdge::CanProvide => Some((space_edge, 0)),
                    _ => None,
                });
            let Some((first_space_edge, first_edge_cost)) = space_edges.next() else {
                unreachable!(
                    "Root node is initialized from the beginning, so should have left the loop at the beginning"
                );
            };

            if let Some((second_space_edge, second_edge_cost)) = space_edges.next() {
                indispensable = false;

                let node_id = self.graph.add_node(());
                self.node_id_to_space_node_id.push(space_node_id);
                self.space_node_id_to_node_id[space_node_id.index()] = node_id;
                if maybe_child_node_id.index() != u32::MAX as usize {
                    self.graph.add_edge(node_id, maybe_child_node_id, maybe_child_edge_cost);
                    self.edge_id_to_space_edge_id.push(maybe_child_space_edge_id);
                }

                let _ = requirements
                    .collect(&space.graph, self, space_node_id)
                    .ingest_as_dispensable(&space.graph, node_id);

                self.nodes_to_process.push(NodeToProcess {
                    space_node_id: first_space_edge.source(),
                    maybe_child_node_id: node_id,
                    maybe_child_space_edge_id: first_space_edge.id(),
                    maybe_child_edge_cost: first_edge_cost,
                });
                self.nodes_to_process.push(NodeToProcess {
                    space_node_id: second_space_edge.source(),
                    maybe_child_node_id: node_id,
                    maybe_child_space_edge_id: second_space_edge.id(),
                    maybe_child_edge_cost: second_edge_cost,
                });

                for (parent_space_edge, parent_edge_cost) in space_edges {
                    self.nodes_to_process.push(NodeToProcess {
                        space_node_id: parent_space_edge.source(),
                        maybe_child_node_id: node_id,
                        maybe_child_space_edge_id: parent_space_edge.id(),
                        maybe_child_edge_cost: parent_edge_cost,
                    });
                }
            } else {
                let requires = requirements.collect(&space.graph, self, space_node_id);
                if indispensable {
                    // Compacting the path, we don't need to keep track of this node since it's
                    // indispensable and the only parent. Cost doesn't matter  either since we need to take
                    // that edge in all cases.
                    let required_space_node_ids = requires.forget_because_indispensable();
                    self.terminals_to_process.extend_from_slice(required_space_node_ids);
                    self.nodes_to_process.push(NodeToProcess {
                        space_node_id: first_space_edge.source(),
                        maybe_child_node_id,
                        maybe_child_space_edge_id,
                        maybe_child_edge_cost,
                    });
                } else if !requires.is_empty() || first_edge_cost > 0 {
                    let node_id = self.graph.add_node(());
                    self.node_id_to_space_node_id.push(space_node_id);
                    self.space_node_id_to_node_id[space_node_id.index()] = node_id;
                    if maybe_child_node_id.index() != u32::MAX as usize {
                        self.graph.add_edge(node_id, maybe_child_node_id, maybe_child_edge_cost);
                        self.edge_id_to_space_edge_id.push(maybe_child_space_edge_id);
                    }

                    let _ = requirements
                        .collect(&space.graph, self, space_node_id)
                        .ingest_as_dispensable(&space.graph, node_id);

                    self.nodes_to_process.push(NodeToProcess {
                        space_node_id: first_space_edge.source(),
                        maybe_child_node_id: node_id,
                        maybe_child_space_edge_id: first_space_edge.id(),
                        maybe_child_edge_cost: first_edge_cost,
                    });
                } else {
                    // There isn't any requirement nor has this edge any cost and it's the only
                    // parent, so we can compact it with the previous node.
                    self.nodes_to_process.push(NodeToProcess {
                        space_node_id: first_space_edge.source(),
                        maybe_child_node_id,
                        maybe_child_space_edge_id,
                        maybe_child_edge_cost,
                    });
                }
            }
        }
    }

    pub fn get_or_insert_node(&mut self, space_node_id: SpaceNodeId) -> NodeIndex {
        let maybe_existing = self.space_node_id_to_node_id[space_node_id.index()];
        if maybe_existing.index() != u32::MAX as usize {
            return maybe_existing;
        }
        let node_ix = self.graph.add_node(());
        self.node_id_to_space_node_id.push(space_node_id);
        self.space_node_id_to_node_id[space_node_id.index()] = node_ix;
        node_ix
    }
}
