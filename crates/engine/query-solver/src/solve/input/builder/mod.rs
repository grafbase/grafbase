mod requirements;

use fxhash::FxHashMap;
use operation::OperationContext;
use petgraph::{
    Direction,
    graph::Graph,
    visit::{EdgeRef, NodeIndexable},
};

use crate::{
    Cost, QuerySolutionSpace, SpaceEdge, SpaceEdgeId, SpaceNodeId,
    solve::input::{InputMap, SteinerGraph, SteinerNodeId, builder::requirements::DispensableRequirementsBuilder},
};
pub(super) struct SteinerInputBuilder<'op> {
    pub ctx: OperationContext<'op>,
    pub graph: SteinerGraph,
    pub root_node_id: SteinerNodeId,
    pub map: InputMap,
    pub terminal_space_node_ids_to_process_stack: Vec<SpaceNodeId>,
    pub nodes_to_process_stack: Vec<NodeToProcess>,
}

pub(super) struct NodeToProcess {
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

impl<'op> SteinerInputBuilder<'op> {
    pub fn build<'schema>(
        ctx: OperationContext<'op>,
        mut space: QuerySolutionSpace<'schema>,
    ) -> (super::SteinerInput<'schema>, Vec<SteinerNodeId>) {
        // TODO: Figure out a good start size.
        let n_nodes = space.graph.node_bound() >> 3;
        let n_edges = space.graph.edge_count() >> 3;
        let mut graph = Graph::with_capacity(n_nodes, n_edges);
        let mut mapping = InputMap {
            node_id_to_space_node_id: Vec::with_capacity(n_nodes),
            edge_id_to_space_edge_id: Vec::with_capacity(n_edges),
            space_node_id_to_node_id: FxHashMap::with_capacity_and_hasher(n_nodes, Default::default()),
            space_edge_id_to_edge_id: FxHashMap::with_capacity_and_hasher(n_edges, Default::default()),
        };

        let root_node_id = graph.add_node(());
        mapping.node_id_to_space_node_id.push(space.root_node_id);
        mapping
            .space_node_id_to_node_id
            .insert(space.root_node_id, root_node_id);

        let mut builder = Self {
            ctx,
            terminal_space_node_ids_to_process_stack: std::mem::take(&mut space.step.indispensable_leaf_nodes),
            graph,
            root_node_id,
            map: mapping,
            nodes_to_process_stack: Vec::new(),
        };
        let mut requirements = DispensableRequirementsBuilder::new(&space.graph);

        let mut terminals = Vec::with_capacity(builder.terminal_space_node_ids_to_process_stack.len());
        while let Some(terminal_space_node_id) = builder.terminal_space_node_ids_to_process_stack.pop() {
            let terminal_node_id =
                builder.ingest_nodes_from_indispensable_terminal(&space, &mut requirements, terminal_space_node_id);
            // If we reached root, it means there is only one path from the root to this terminal.
            if terminal_node_id != root_node_id {
                terminals.push(terminal_node_id);
            }
        }

        terminals.sort_unstable();
        terminals.dedup();

        let requirements = requirements.build(&mut builder);
        let input = super::SteinerInput {
            space,
            graph: builder.graph,
            root_node_id: builder.root_node_id,
            map: builder.map,
            requirements,
        };
        (input, terminals)
    }

    fn ingest_nodes_from_indispensable_terminal(
        &mut self,
        space: &QuerySolutionSpace<'_>,
        requirements: &mut DispensableRequirementsBuilder,
        terminal_space_node_id: SpaceNodeId,
    ) -> SteinerNodeId {
        let mut terminal_node_id = None;
        debug_assert!(self.nodes_to_process_stack.is_empty());
        self.nodes_to_process_stack
            .push(NodeToProcess::terminal(terminal_space_node_id));

        while let Some(NodeToProcess {
            space_node_id,
            maybe_child_node_id,
            maybe_child_space_edge_id,
            maybe_child_edge_cost,
        }) = self.nodes_to_process_stack.pop()
        {
            tracing::debug!("Processing node: {}", space.graph[space_node_id].label(space, self.ctx));
            // Already processed, just add the edge.
            if let Some(&existing_node_id) = self.map.space_node_id_to_node_id.get(&space_node_id) {
                tracing::debug!(
                    "Found existing node: {}",
                    space.graph[self.map.node_id_to_space_node_id[existing_node_id.index()]].label(space, self.ctx)
                );
                if terminal_node_id.is_none() {
                    terminal_node_id = Some(existing_node_id);
                }
                if maybe_child_node_id.index() != u32::MAX as usize {
                    self.graph
                        .add_edge(existing_node_id, maybe_child_node_id, maybe_child_edge_cost);
                    self.map.edge_id_to_space_edge_id.push(maybe_child_space_edge_id);
                }
                continue;
            }

            let mut space_edges = space
                .graph
                .edges_directed(space_node_id, Direction::Incoming)
                .filter_map(|space_edge| match space_edge.weight() {
                    SpaceEdge::CreateChildResolver => Some((space_edge, 1)),
                    SpaceEdge::CanProvide | SpaceEdge::Provides => Some((space_edge, 0)),
                    _ => None,
                });
            let Some((first_space_edge, first_edge_cost)) = space_edges.next() else {
                tracing::debug!("Node without parent edges",);
                unreachable!(
                    "Root node is initialized from the beginning, so should have left the loop at the beginning"
                );
            };

            if let Some((second_space_edge, second_edge_cost)) = space_edges.next() {
                tracing::debug!("Multiple edges");
                let node_id = self.graph.add_node(());
                self.map.node_id_to_space_node_id.push(space_node_id);
                self.map.space_node_id_to_node_id.insert(space_node_id, node_id);
                if maybe_child_node_id.index() != u32::MAX as usize {
                    let edge_id = self.graph.add_edge(node_id, maybe_child_node_id, maybe_child_edge_cost);
                    self.map.edge_id_to_space_edge_id.push(maybe_child_space_edge_id);
                    self.map
                        .space_edge_id_to_edge_id
                        .insert(maybe_child_space_edge_id, edge_id);
                }

                if terminal_node_id.is_none() {
                    terminal_node_id = Some(node_id);
                    requirements
                        .collect(&space.graph, space_node_id)
                        .forget_because_indispensable(|required_space_node_ids| {
                            self.terminal_space_node_ids_to_process_stack
                                .extend_from_slice(required_space_node_ids);
                        });
                } else {
                    requirements
                        .collect(&space.graph, space_node_id)
                        .ingest_as_dispensable(&space.graph, node_id);
                }

                self.nodes_to_process_stack.push(NodeToProcess {
                    space_node_id: first_space_edge.source(),
                    maybe_child_node_id: node_id,
                    maybe_child_space_edge_id: first_space_edge.id(),
                    maybe_child_edge_cost: first_edge_cost,
                });
                self.nodes_to_process_stack.push(NodeToProcess {
                    space_node_id: second_space_edge.source(),
                    maybe_child_node_id: node_id,
                    maybe_child_space_edge_id: second_space_edge.id(),
                    maybe_child_edge_cost: second_edge_cost,
                });

                for (parent_space_edge, parent_edge_cost) in space_edges {
                    self.nodes_to_process_stack.push(NodeToProcess {
                        space_node_id: parent_space_edge.source(),
                        maybe_child_node_id: node_id,
                        maybe_child_space_edge_id: parent_space_edge.id(),
                        maybe_child_edge_cost: parent_edge_cost,
                    });
                }
            } else {
                let requires = requirements.collect(&space.graph, space_node_id);
                if terminal_node_id.is_none() {
                    // Compacting the path, we don't need to keep track of this node since it's
                    // indispensable and the only parent. Cost doesn't matter  either since we need to take
                    // that edge in all cases.
                    requires.forget_because_indispensable(|required_space_node_ids| {
                        self.terminal_space_node_ids_to_process_stack
                            .extend_from_slice(required_space_node_ids);
                    });

                    self.nodes_to_process_stack.push(NodeToProcess {
                        space_node_id: first_space_edge.source(),
                        maybe_child_node_id,
                        maybe_child_space_edge_id,
                        maybe_child_edge_cost,
                    });
                } else if !requires.is_empty() || first_edge_cost > 0 {
                    tracing::debug!("Has requirements or non-empty cost.");
                    let node_id = self.graph.add_node(());
                    self.map.node_id_to_space_node_id.push(space_node_id);
                    self.map.space_node_id_to_node_id.insert(space_node_id, node_id);
                    if maybe_child_node_id.index() != u32::MAX as usize {
                        let edge_id = self.graph.add_edge(node_id, maybe_child_node_id, maybe_child_edge_cost);
                        self.map.edge_id_to_space_edge_id.push(maybe_child_space_edge_id);
                        self.map
                            .space_edge_id_to_edge_id
                            .insert(maybe_child_space_edge_id, edge_id);
                    }

                    requires.ingest_as_dispensable(&space.graph, node_id);

                    self.nodes_to_process_stack.push(NodeToProcess {
                        space_node_id: first_space_edge.source(),
                        maybe_child_node_id: node_id,
                        maybe_child_space_edge_id: first_space_edge.id(),
                        maybe_child_edge_cost: first_edge_cost,
                    });
                } else {
                    tracing::debug!("Single parent.");
                    // There isn't any requirement nor has this edge any cost and it's the only
                    // parent, so we can compact it with the previous node.
                    self.nodes_to_process_stack.push(NodeToProcess {
                        space_node_id: first_space_edge.source(),
                        maybe_child_node_id,
                        maybe_child_space_edge_id,
                        maybe_child_edge_cost,
                    });
                }
            }
        }

        terminal_node_id.expect("Terminal node should be set at the end of the loop")
    }
}
