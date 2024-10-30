use crate::{Edge, Node, Operation};
use petgraph::{
    visit::{EdgeRef, IntoNodeReferences},
    Direction,
};

use super::builder::OperationGraphBuilder;

impl<'ctx, Op: Operation> OperationGraphBuilder<'ctx, Op> {
    pub(super) fn prune_resolvers_not_leading_to_any_scalar_node(&mut self) {
        let mut included = fixedbitset::FixedBitSet::with_capacity(self.graph.node_count());
        let mut stack = self
            .graph
            .node_references()
            .filter_map(|(node_ix, node)| match node {
                Node::QueryField(field) if field.is_scalar() => Some(node_ix),
                _ => None,
            })
            .collect::<Vec<_>>();

        while let Some(node) = stack.pop() {
            if included[node.index()] {
                continue;
            };
            stack.extend(
                self.graph
                    .edges_directed(node, Direction::Incoming)
                    .filter(|edge| match edge.weight() {
                        Edge::CreateChildResolver { .. } | Edge::CanProvide { .. } | Edge::Provides => true,
                        Edge::Field | Edge::TypenameField | Edge::Requires { .. } | Edge::HasChildResolver { .. } => {
                            false
                        }
                    })
                    .map(|edge| edge.source()),
            );
            included.set(node.index(), true);
        }

        self.graph.retain_nodes(|graph, ix| match graph[ix] {
            Node::Root | Node::QueryField(_) => true,
            Node::Resolver(_) | Node::ProvidableField(_) => included[ix.index()],
        });
    }
}
