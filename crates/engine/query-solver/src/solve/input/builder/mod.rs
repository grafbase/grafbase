mod requirements;

use fxhash::FxHashMap;
use operation::OperationContext;
use petgraph::{
    Direction,
    graph::Graph,
    visit::{EdgeRef, NodeIndexable},
};

use crate::{
    QuerySolutionSpace, SpaceEdge, SpaceEdgeId, SpaceNodeId,
    solve::input::{
        InputMap, SteinerGraph, SteinerNodeId, SteinerWeight, builder::requirements::DispensableRequirementsBuilder,
    },
};
pub(super) struct SteinerInputBuilder<'op> {
    // Useful for debugging and tracing.
    #[allow(unused)]
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
    maybe_child_edge_weight: SteinerWeight,
}

impl NodeToProcess {
    fn terminal(space_node_id: SpaceNodeId) -> Self {
        Self {
            space_node_id,
            maybe_child_node_id: SteinerNodeId::new(u32::MAX as usize),
            maybe_child_space_edge_id: SpaceEdgeId::new(u32::MAX as usize),
            maybe_child_edge_weight: 0,
        }
    }
}

pub(crate) fn build_input_and_terminals<'op, 'schema>(
    ctx: OperationContext<'op>,
    mut space: QuerySolutionSpace<'schema>,
) -> crate::Result<(super::SteinerInput<'schema>, Vec<SteinerNodeId>)> {
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

    let mut builder = SteinerInputBuilder {
        ctx,
        terminal_space_node_ids_to_process_stack: std::mem::take(&mut space.step.indispensable_leaf_nodes),
        graph,
        root_node_id,
        map: mapping,
        nodes_to_process_stack: Vec::new(),
    };
    let mut requirements = DispensableRequirementsBuilder::new(&space.graph);

    let mut terminals = Vec::with_capacity(builder.terminal_space_node_ids_to_process_stack.len());
    let mut i = 0;
    while let Some(terminal_space_node_id) = builder.terminal_space_node_ids_to_process_stack.pop() {
        // Sanity check to prevent infinite loops. At most we can have as many terminals as
        // we have leaf nodes. With requirements we may have duplicates, hence the dedup
        // afterwards, but in all cases we cannot have more terminals than edges in the
        // graph.
        // We could be smarter, but a properly composed schema should prevent cycles from being
        // there in the first place.
        if i > space.graph.edge_count() {
            return Err(crate::Error::RequirementCycleDetected);
        }
        i += 1;

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
    Ok((input, terminals))
}

impl SteinerInputBuilder<'_> {
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
            maybe_child_edge_weight,
        }) = self.nodes_to_process_stack.pop()
        {
            // Already processed, just add the edge.
            if let Some(&existing_node_id) = self.map.space_node_id_to_node_id.get(&space_node_id) {
                if terminal_node_id.is_none() {
                    terminal_node_id = Some(existing_node_id);
                }
                if maybe_child_node_id.index() != u32::MAX as usize {
                    self.graph
                        .add_edge(existing_node_id, maybe_child_node_id, maybe_child_edge_weight);
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
            let Some((first_space_edge, first_edge_weight)) = space_edges.next() else {
                unreachable!(
                    "Root node is initialized from the beginning, so should have left the loop at the beginning"
                );
            };

            if let Some((second_space_edge, second_edge_weight)) = space_edges.next() {
                let node_id = self.graph.add_node(());
                self.map.node_id_to_space_node_id.push(space_node_id);
                self.map.space_node_id_to_node_id.insert(space_node_id, node_id);
                if maybe_child_node_id.index() != u32::MAX as usize {
                    let edge_id = self
                        .graph
                        .add_edge(node_id, maybe_child_node_id, maybe_child_edge_weight);
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
                    maybe_child_edge_weight: first_edge_weight,
                });
                self.nodes_to_process_stack.push(NodeToProcess {
                    space_node_id: second_space_edge.source(),
                    maybe_child_node_id: node_id,
                    maybe_child_space_edge_id: second_space_edge.id(),
                    maybe_child_edge_weight: second_edge_weight,
                });

                for (parent_space_edge, parent_edge_weight) in space_edges {
                    self.nodes_to_process_stack.push(NodeToProcess {
                        space_node_id: parent_space_edge.source(),
                        maybe_child_node_id: node_id,
                        maybe_child_space_edge_id: parent_space_edge.id(),
                        maybe_child_edge_weight: parent_edge_weight,
                    });
                }
            } else {
                let requires = requirements.collect(&space.graph, space_node_id);
                if terminal_node_id.is_none() {
                    // Compacting the path, we don't need to keep track of this node since it's
                    // indispensable and the only parent. weight doesn't matter  either since we need to take
                    // that edge in all cases.
                    requires.forget_because_indispensable(|required_space_node_ids| {
                        self.terminal_space_node_ids_to_process_stack
                            .extend_from_slice(required_space_node_ids);
                    });

                    self.nodes_to_process_stack.push(NodeToProcess {
                        space_node_id: first_space_edge.source(),
                        maybe_child_node_id,
                        maybe_child_space_edge_id,
                        maybe_child_edge_weight,
                    });
                } else if !requires.is_empty() || first_edge_weight > 0 {
                    let node_id = self.graph.add_node(());
                    self.map.node_id_to_space_node_id.push(space_node_id);
                    self.map.space_node_id_to_node_id.insert(space_node_id, node_id);
                    if maybe_child_node_id.index() != u32::MAX as usize {
                        let edge_id = self
                            .graph
                            .add_edge(node_id, maybe_child_node_id, maybe_child_edge_weight);
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
                        maybe_child_edge_weight: first_edge_weight,
                    });
                } else {
                    // There isn't any requirement nor has this edge any weight and it's the only
                    // parent, so we can compact it with the previous node.
                    self.nodes_to_process_stack.push(NodeToProcess {
                        space_node_id: first_space_edge.source(),
                        maybe_child_node_id,
                        maybe_child_space_edge_id,
                        maybe_child_edge_weight,
                    });
                }
            }
        }

        terminal_node_id.expect("Terminal node should be set at the end of the loop")
    }
}
