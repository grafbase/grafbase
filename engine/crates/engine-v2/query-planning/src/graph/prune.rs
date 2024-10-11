use crate::{Edge, Node, Operation};
use bitvec::bitvec;
use petgraph::{visit::EdgeRef, Direction};
use schema::Definition;
use walker::Walk;

use super::builder::OperationGraphBuilder;

impl<'ctx, Op: Operation> OperationGraphBuilder<'ctx, Op> {
    pub(super) fn prune_resolvers_not_leading_to_any_scalar_node(&mut self) {
        let mut included = bitvec![false as usize; self.graph.node_count()];
        let mut stack = self
            .operation
            .field_ids()
            .filter(|field_id| {
                self.operation.field_defintion(*field_id).is_some_and(|definition| {
                    matches!(definition.walk(self.schema).ty().definition(), Definition::Scalar(_))
                })
            })
            .map(|field_id| self[field_id])
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
                        Edge::Field | Edge::TypenameField | Edge::Requires { .. } | Edge::HasChildResolver => false,
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
