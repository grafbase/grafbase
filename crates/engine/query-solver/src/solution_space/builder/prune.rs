use petgraph::{
    visit::{EdgeRef, IntoNodeReferences, NodeIndexable},
    Direction,
};

use crate::FieldFlags;

use super::{builder::QuerySolutionSpaceBuilder, SpaceEdge, SpaceNode};

impl QuerySolutionSpaceBuilder<'_, '_> {
    pub(super) fn prune_resolvers_not_leading_any_leafs(&mut self) {
        let mut visited = fixedbitset::FixedBitSet::with_capacity(self.query.graph.node_bound());
        let mut stack = Vec::new();
        let mut extra_leafs = Vec::new();

        for (node_ix, node) in self.query.graph.node_references() {
            let SpaceNode::QueryField(node) = node else {
                continue;
            };
            // Any resolver that can eventually provide a scalar/__typename must be kept
            if node.is_leaf() {
                stack.push(node_ix);
                if self.query[node.id].definition_id.is_none() {
                    let Some(parent_edge) =
                        self.query
                            .graph
                            .edges_directed(node_ix, Direction::Incoming)
                            .find(|edge| {
                                matches!(edge.weight(), SpaceEdge::TypenameField)
                                    && matches!(self.query.graph[edge.source()], SpaceNode::QueryField(_))
                            })
                    else {
                        continue;
                    };
                    // If the parent node only provides __typenames
                    let parent_node_ix = parent_edge.source();
                    if self
                        .query
                        .graph
                        .edges(parent_node_ix)
                        .filter(|edge| matches!(edge.weight(), SpaceEdge::Field))
                        .count()
                        == 0
                    {
                        extra_leafs.push(parent_node_ix);
                        stack.push(parent_node_ix);
                    }
                }
            }
        }

        for leaf_node_ix in extra_leafs {
            let SpaceNode::QueryField(field) = &mut self.query.graph[leaf_node_ix] else {
                continue;
            };
            // If dispensable, how could it have a __typename?
            debug_assert!(field.flags.contains(FieldFlags::INDISPENSABLE));
            field.flags |= FieldFlags::LEAF_NODE;
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
                            | SpaceEdge::TypenameField
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
