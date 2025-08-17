mod requirements;

use fixedbitset::FixedBitSet;
use fxhash::FxHashMap;
use operation::OperationContext;
use petgraph::{
    Direction,
    graph::Graph,
    visit::{EdgeIndexable as _, EdgeRef, IntoNodeReferences, NodeIndexable},
};

use crate::{
    FieldFlags, QuerySolutionSpace, SpaceEdge, SpaceEdgeId, SpaceNode, SpaceNodeId,
    solve::{
        input::{
            SteinerGraph, SteinerInputMap, SteinerNodeId, SteinerWeight,
            builder::requirements::DispensableRequirementsBuilder,
        },
        steiner_tree::SteinerTree,
    },
};

const RESOLVER_BASE_WEIGHT: SteinerWeight = 10;
const DEPTH_WEIGHT: SteinerWeight = 1;

pub(super) struct SteinerInputBuilder<'schema, 'op, 'space> {
    pub space: &'space QuerySolutionSpace<'schema>,
    // Useful for debugging and tracing.
    #[allow(unused)]
    pub ctx: OperationContext<'op>,
    pub graph: SteinerGraph,
    pub root_node_id: SteinerNodeId,
    pub map: SteinerInputMap,
    pub indispensable_terminal_space_node_ids: Vec<SpaceNodeId>,
    pub dispensable_terminal_space_node_ids: Vec<SpaceNodeId>,
    pub nodes_to_process_stack: Vec<NodeToProcess>,
    pub space_node_path_stack: Vec<SpaceNodeId>,
    pub space_node_relative_depth_weight_to_steiner_node: Vec<SteinerWeight>,
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

pub(crate) fn build_input_and_terminals<'schema, 'op>(
    ctx: OperationContext<'op>,
    space: QuerySolutionSpace<'schema>,
) -> crate::Result<(super::SteinerInput<'schema>, SteinerTree)> {
    let n_nodes = space.graph.node_count() >> 3;
    let n_edges = space.graph.edge_count() >> 3;
    let mut graph = Graph::with_capacity(n_nodes, n_edges);
    let mut mapping = SteinerInputMap {
        node_id_to_space_node_id: Vec::with_capacity(n_nodes),
        edge_id_to_space_edge_id: Vec::with_capacity(n_edges),
        space_node_id_to_node_id: vec![SpaceNodeId::new(u32::MAX as usize); space.graph.node_bound()],
        space_edge_id_to_edge_id: FxHashMap::with_capacity_and_hasher(n_edges, Default::default()),
    };

    let root_node_id = graph.add_node(());
    mapping.node_id_to_space_node_id.push(space.root_node_id);
    mapping.space_node_id_to_node_id[space.root_node_id.index()] = root_node_id;

    let indispensable_terminal_space_node_ids = space
        .graph
        .node_references()
        .filter_map(|(node_id, node)| match node {
            SpaceNode::QueryField(field) if field.flags.contains(FieldFlags::INDISPENSABLE | FieldFlags::LEAF_NODE) => {
                Some(node_id)
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    // TODO: We could sort the terminals by least amount of incoming providable fields to ensure we
    // compact as much as possible.
    let mut builder = SteinerInputBuilder {
        space: &space,
        ctx,
        indispensable_terminal_space_node_ids,
        dispensable_terminal_space_node_ids: Vec::new(),
        graph,
        root_node_id,
        map: mapping,
        nodes_to_process_stack: Vec::new(),
        space_node_path_stack: Vec::new(),
        space_node_relative_depth_weight_to_steiner_node: vec![0; space.graph.node_bound()],
    };
    let mut requirements = DispensableRequirementsBuilder::new(&space.graph);

    let mut tree = SteinerTree {
        nodes: FixedBitSet::new(),
        edges: FixedBitSet::new(),
        total_weight: 0,
        terminals: Vec::with_capacity(builder.dispensable_terminal_space_node_ids.len()),
        is_terminal: FixedBitSet::with_capacity(builder.graph.node_bound()),
    };
    tree.is_terminal.insert(builder.root_node_id.index());

    let mut space_node_is_terminal = FixedBitSet::with_capacity(space.graph.node_bound());
    let mut i = 0;
    while let Some(terminal_space_node_id) = builder.indispensable_terminal_space_node_ids.pop() {
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

        space_node_is_terminal.insert(terminal_space_node_id.index());
        let terminal_node_id = builder.ingest_nodes_from_terminal(&mut requirements, terminal_space_node_id, true);
        // If we reached root, it means there is only one path from the root to this terminal.
        if terminal_node_id != root_node_id && !tree.is_terminal[terminal_node_id.index()] {
            tree.is_terminal.grow_and_insert(terminal_node_id.index());
            tree.terminals.push(terminal_node_id);
        }
    }

    while let Some(space_node_id) = builder.dispensable_terminal_space_node_ids.pop() {
        builder.ingest_nodes_from_terminal(&mut requirements, space_node_id, false);
    }

    // Finalize Steiner tree initialization.
    tree.nodes = FixedBitSet::with_capacity(builder.graph.node_bound());
    tree.nodes.insert(root_node_id.index());
    tree.edges = FixedBitSet::with_capacity(builder.graph.edge_bound());
    tree.is_terminal.grow(builder.graph.node_bound());

    let requirements = requirements.build(&builder, &tree);
    let SteinerInputBuilder {
        graph,
        root_node_id,
        map,
        ..
    } = builder;

    let input = super::SteinerInput {
        space,
        space_node_is_terminal,
        graph,
        root_node_id,
        map,
        requirements,
    };
    Ok((input, tree))
}

impl SteinerInputBuilder<'_, '_, '_> {
    fn ingest_nodes_from_terminal(
        &mut self,
        requirements: &mut DispensableRequirementsBuilder,
        terminal_space_node_id: SpaceNodeId,
        indispensable: bool,
    ) -> SteinerNodeId {
        let mut terminal_node_id = if indispensable {
            None
        } else {
            Some(SteinerNodeId::new(u32::MAX as usize))
        };
        debug_assert!(self.nodes_to_process_stack.is_empty() && self.space_node_path_stack.is_empty());
        self.nodes_to_process_stack
            .push(NodeToProcess::terminal(terminal_space_node_id));

        // We're effectively implementing a backtracking algorithm with stacks rather than the
        // function stack.
        while let Some(NodeToProcess {
            space_node_id,
            maybe_child_node_id,
            maybe_child_space_edge_id,
            maybe_child_edge_weight,
        }) = self.nodes_to_process_stack.pop()
        {
            // Already processed, just add the edge.
            let maybe_existing_node_id = self.map.space_node_id_to_node_id[space_node_id.index()];
            if maybe_existing_node_id.index() != u32::MAX as usize {
                if terminal_node_id.is_none() {
                    terminal_node_id = Some(maybe_existing_node_id);
                }
                self.handle_children(
                    maybe_existing_node_id,
                    NodeToProcess {
                        space_node_id,
                        maybe_child_node_id,
                        maybe_child_space_edge_id,
                        maybe_child_edge_weight,
                    },
                );
                continue;
            }

            let mut space_edges = self
                .space
                .graph
                .edges_directed(space_node_id, Direction::Incoming)
                .filter_map(|space_edge| match space_edge.weight() {
                    SpaceEdge::CreateChildResolver => Some((space_edge, RESOLVER_BASE_WEIGHT)),
                    SpaceEdge::CanProvide | SpaceEdge::Provides => Some((space_edge, 0)),
                    _ => None,
                });
            let Some((first_space_edge, first_edge_weight)) = space_edges.next() else {
                unreachable!(
                    "Root node is initialized from the beginning, so should have left the loop at the beginning: {}",
                    self.space.graph[space_node_id].label(self.space, self.ctx)
                );
            };

            if let Some((second_space_edge, second_edge_weight)) = space_edges.next() {
                let node_id = self.add(NodeToProcess {
                    space_node_id,
                    maybe_child_node_id,
                    maybe_child_space_edge_id,
                    maybe_child_edge_weight,
                });

                if terminal_node_id.is_none() {
                    terminal_node_id = Some(node_id);
                    requirements
                        .collect(&self.space.graph, space_node_id)
                        .forget_because_indispensable(|required_space_node_ids| {
                            self.indispensable_terminal_space_node_ids
                                .extend_from_slice(required_space_node_ids);
                        });
                } else {
                    self.dispensable_terminal_space_node_ids.extend_from_slice(
                        requirements
                            .collect(&self.space.graph, space_node_id)
                            .ingest_as_dispensable(&self.space.graph, node_id),
                    );
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
                let requires = requirements.collect(&self.space.graph, space_node_id);
                if terminal_node_id.is_none() {
                    // Compacting the path, we don't need to keep track of this node since it's
                    // indispensable and the only parent. weight doesn't matter  either since we need to take
                    // that edge in all cases.
                    requires.forget_because_indispensable(|required_space_node_ids| {
                        self.indispensable_terminal_space_node_ids
                            .extend_from_slice(required_space_node_ids);
                    });

                    self.space_node_path_stack.push(space_node_id);
                    // should not have any child since we have yet to map the terminal to any node.
                    debug_assert!(maybe_child_node_id == SpaceNodeId::new(u32::MAX as usize));
                    self.nodes_to_process_stack.push(NodeToProcess {
                        space_node_id: first_space_edge.source(),
                        maybe_child_node_id,
                        maybe_child_space_edge_id,
                        maybe_child_edge_weight,
                    });
                } else if !requires.is_empty() || first_edge_weight > 0 {
                    let node_id = self.add(NodeToProcess {
                        space_node_id,
                        maybe_child_node_id,
                        maybe_child_space_edge_id,
                        maybe_child_edge_weight,
                    });

                    self.dispensable_terminal_space_node_ids
                        .extend_from_slice(requires.ingest_as_dispensable(&self.space.graph, node_id));

                    self.nodes_to_process_stack.push(NodeToProcess {
                        space_node_id: first_space_edge.source(),
                        maybe_child_node_id: node_id,
                        maybe_child_space_edge_id: first_space_edge.id(),
                        maybe_child_edge_weight: first_edge_weight,
                    });
                } else {
                    // There isn't any requirement nor has this edge any weight and it's the only
                    // parent, so we can compact it with the previous node.
                    self.space_node_path_stack.push(space_node_id);
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

    fn add(
        &mut self,
        NodeToProcess {
            space_node_id,
            maybe_child_node_id,
            maybe_child_space_edge_id,
            maybe_child_edge_weight,
        }: NodeToProcess,
    ) -> SteinerNodeId {
        let node_id = self.add_node(space_node_id);
        self.handle_children(
            node_id,
            NodeToProcess {
                space_node_id,
                maybe_child_node_id,
                maybe_child_space_edge_id,
                maybe_child_edge_weight,
            },
        );

        node_id
    }

    fn handle_children(
        &mut self,
        node_id: SteinerNodeId,
        NodeToProcess {
            space_node_id,
            maybe_child_node_id,
            maybe_child_space_edge_id,
            maybe_child_edge_weight,
        }: NodeToProcess,
    ) {
        let mut weight = 0;
        while let Some(space_node_id) = self.space_node_path_stack.pop() {
            self.map.space_node_id_to_node_id[space_node_id.index()] = node_id;
            let is_resolver = matches!(self.space.graph[space_node_id], SpaceNode::Resolver(_));
            weight += is_resolver as SteinerWeight * DEPTH_WEIGHT;
            self.space_node_relative_depth_weight_to_steiner_node[space_node_id.index()] = weight;
        }
        if maybe_child_node_id.index() != u32::MAX as usize {
            let relative_weight = self.space_node_relative_depth_weight_to_steiner_node[space_node_id.index()];
            self.add_edge(
                node_id,
                maybe_child_node_id,
                maybe_child_edge_weight + relative_weight,
                maybe_child_space_edge_id,
            );
        }
    }

    fn add_node(&mut self, space_node_id: SpaceNodeId) -> SteinerNodeId {
        let node_id = self.graph.add_node(());
        self.map.node_id_to_space_node_id.push(space_node_id);
        self.map.space_node_id_to_node_id[space_node_id.index()] = node_id;
        node_id
    }

    fn add_edge(
        &mut self,
        source_node_id: SteinerNodeId,
        target_node_id: SteinerNodeId,
        weight: SteinerWeight,
        space_edge_id: SpaceEdgeId,
    ) {
        let edge_id = self.graph.add_edge(source_node_id, target_node_id, weight);
        self.map.edge_id_to_space_edge_id.push(space_edge_id);
        self.map.space_edge_id_to_edge_id.insert(space_edge_id, edge_id);
    }
}
