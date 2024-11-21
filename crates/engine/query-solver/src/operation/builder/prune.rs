use petgraph::{
    visit::{EdgeRef, IntoNodeReferences, NodeIndexable},
    Direction,
};

use crate::FieldFlags;

use super::{builder::OperationGraphBuilder, Edge, Node, Operation};

impl<Op: Operation> OperationGraphBuilder<'_, Op> {
    pub(super) fn prune_resolvers_not_leading_any_leafs(&mut self) {
        let mut visited = fixedbitset::FixedBitSet::with_capacity(self.graph.node_bound());

        let mut stack = Vec::new();
        let mut extra_leafs = Vec::new();

        for (node_ix, node) in self.graph.node_references() {
            let Node::QueryField(field) = node else {
                continue;
            };
            // Any resolver that can provide a scalar must be kept
            if field.is_leaf() {
                stack.push(node_ix);
            } else if field.is_typename() {
                let Some(parent_edge) = self.graph.edges_directed(node_ix, Direction::Incoming).find(|edge| {
                    matches!(edge.weight(), Edge::TypenameField)
                        && matches!(self.graph[edge.source()], Node::QueryField(_))
                }) else {
                    continue;
                };
                // If the parent node only provides __typenames
                let parent_node = parent_edge.source();
                if self
                    .graph
                    .edges(parent_node)
                    .filter(|edge| matches!(edge.weight(), Edge::Field))
                    .count()
                    == 0
                {
                    extra_leafs.push(parent_node);
                    stack.push(parent_node);
                }
            }
        }

        for leaf_node_ix in extra_leafs {
            let Node::QueryField(field) = &mut self.graph[leaf_node_ix] else {
                continue;
            };
            // If dispensable, how could it have a __typename?
            debug_assert!(field.flags.contains(FieldFlags::INDISPENSABLE));
            field.flags |= FieldFlags::LEAF_NODE;
        }

        while let Some(node) = stack.pop() {
            if !visited.put(node.index()) {
                stack.extend(
                    self.graph
                        .edges_directed(node, Direction::Incoming)
                        .filter(|edge| match edge.weight() {
                            Edge::CreateChildResolver { .. } | Edge::CanProvide { .. } | Edge::Provides => true,
                            Edge::Field
                            | Edge::TypenameField
                            | Edge::Requires { .. }
                            | Edge::HasChildResolver { .. } => false,
                        })
                        .map(|edge| edge.source()),
                );
            };
        }

        self.graph.retain_nodes(|graph, ix| match graph[ix] {
            Node::Root | Node::QueryField(_) => true,
            Node::Resolver(_) | Node::ProvidableField(_) => visited[ix.index()],
        });
    }
}
