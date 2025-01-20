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
        let mut extra_leafs = Vec::new();

        for (node_ix, node) in self.query.graph.node_references() {
            if let SpaceNode::QueryField {
                flags,
                typename_node_ix,
                ..
            } = node
            {
                if flags.contains(NodeFlags::LEAF) {
                    stack.push(node_ix);
                } else if !self
                    .query
                    .graph
                    .edges(node_ix)
                    .any(|edge| matches!(edge.weight(), SpaceEdge::Field))
                {
                    let extra_leaf_ix = (*typename_node_ix).unwrap_or(node_ix);
                    stack.push(extra_leaf_ix);
                    extra_leafs.push(extra_leaf_ix);
                }
            }
        }

        for extra_leaf in extra_leafs {
            if let Some(flags) = self.query.graph[extra_leaf].flags_mut() {
                flags.insert(NodeFlags::LEAF);
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
