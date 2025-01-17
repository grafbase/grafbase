use petgraph::{
    visit::{EdgeRef, IntoNodeReferences, NodeIndexable},
    Direction,
};

use crate::NodeFlags;

use super::{builder::QuerySolutionSpaceBuilder, SpaceEdge, SpaceNode};

impl QuerySolutionSpaceBuilder<'_, '_> {
    pub(super) fn prune_resolvers_not_leading_any_leafs(&mut self) {
        let mut visited = fixedbitset::FixedBitSet::with_capacity(self.query.graph.node_bound());
        let mut stack = Vec::new();

        for (node_ix, node) in self.query.graph.node_references() {
            match node {
                SpaceNode::QueryField { flags, .. } if flags.contains(NodeFlags::LEAF) => {
                    stack.push(node_ix);
                }
                _ => (),
            }
        }

        for selection_set in &self.query.selection_sets {
            if self
                .query
                .graph
                .edges(selection_set.parent_node_ix)
                .filter(|edge| matches!(edge.weight(), SpaceEdge::Field))
                .count()
                == 0
            {
                let leaf_node_ix =
                    if let Some((typename_node_ix, _)) = selection_set.typename_node_ix_and_petitioner_location {
                        typename_node_ix
                    } else {
                        selection_set.parent_node_ix
                    };
                if let Some(flags) = self.query.graph[leaf_node_ix].flags_mut() {
                    flags.insert(NodeFlags::LEAF);
                }
                stack.push(leaf_node_ix);
            }
        }

        while let Some(node) = stack.pop() {
            if !visited.put(node.index()) {
                stack.extend(
                    self.query
                        .graph
                        .edges_directed(node, Direction::Incoming)
                        .filter(|edge| match edge.weight() {
                            SpaceEdge::CreateChildResolver { .. }
                            | SpaceEdge::CanProvide { .. }
                            | SpaceEdge::Provides
                            | SpaceEdge::ProvidesTypename => true,
                            SpaceEdge::Field
                            | SpaceEdge::RequiredBySupergraph { .. }
                            | SpaceEdge::RequiredBySubgraph { .. }
                            | SpaceEdge::HasChildResolver { .. }
                            | SpaceEdge::TypenameField => false,
                        })
                        .map(|edge| edge.source()),
                );
            };
        }

        self.query.graph.retain_nodes(|graph, ix| match graph[ix] {
            SpaceNode::Root | SpaceNode::QueryField { .. } | SpaceNode::Typename { .. } => true,
            SpaceNode::Resolver(_) | SpaceNode::ProvidableField(_) => visited[ix.index()],
        });
    }
}
