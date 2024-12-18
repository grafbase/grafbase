use petgraph::{
    visit::{EdgeRef, IntoNodeReferences, NodeIndexable},
    Direction,
};

use super::{builder::QuerySolutionSpaceBuilder, SpaceEdge, SpaceNode};

impl QuerySolutionSpaceBuilder<'_, '_> {
    pub(super) fn prune_resolvers_not_leading_any_leafs(&mut self) {
        let mut visited = fixedbitset::FixedBitSet::with_capacity(self.query.graph.node_bound());
        let mut stack = Vec::new();

        for (node_ix, node) in self.query.graph.node_references() {
            let SpaceNode::QueryField(field) = node else {
                continue;
            };
            // Any resolver that can eventually provide a scalar/__typename must be kept
            if field.is_leaf() {
                stack.push(node_ix);
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
                            | SpaceEdge::Provides => true,
                            SpaceEdge::Field | SpaceEdge::Requires { .. } | SpaceEdge::HasChildResolver { .. } => false,
                        })
                        .map(|edge| edge.source()),
                );
            };
        }

        self.query.graph.retain_nodes(|graph, ix| match graph[ix] {
            SpaceNode::Root | SpaceNode::QueryField(_) => true,
            SpaceNode::Resolver(_) | SpaceNode::ProvidableField(_) => visited[ix.index()],
        });
    }
}
