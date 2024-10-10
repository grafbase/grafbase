use petgraph::{
    graph::NodeIndex,
    visit::{EdgeRef, IntoNodeReferences},
    Direction,
};
use tracing::instrument;

use crate::{Edge, Node, Operation, OperationGraph};

const MAX_ITERATIONS: usize = 100;

impl<'ctx, Op: Operation> OperationGraph<'ctx, Op> {
    /// Gross approximation of the real cost of resolvers and fields. For any requirement we
    /// compute the cost as the number of additional resolvers we need to go through.
    ///
    /// Supposing we the following fields: `{ a { b { c } e } }` and `e` depends on `c`. We'll roughly
    /// generate the following operation graph: `a -> b -> c` and `a -> e`. We'll find their common
    /// ancestor `a`, and from there the cost of `c` for `e` is the cost of `b` and `c`.
    ///
    /// As changing the cost of a resolver may impact any dependent ones, we need to iterate
    /// several times to compute their real cost. If there is no cycles, which
    /// composition should ensure, we will end up in a finite number of iterations until
    /// convergence. But how many is unclear and a composition error shouldn't lead to an infinite
    /// loop, so we're capping the max number of iterations to an arbitrary seemingly large enough
    /// number.
    ///
    /// Today each iterations goes naively over all nodes again, we can certainly be smarter than
    /// that.
    #[instrument(skip_all)]
    #[allow(unused)]
    pub(crate) fn estimate_resolver_costs(&mut self) {
        let nodes = self
            .graph
            .node_references()
            .filter_map(|(ix, weight)| match weight {
                Node::Resolver(_) | Node::FieldResolver(_)
                    if self
                        .graph
                        .edges_directed(ix, Direction::Outgoing)
                        .any(|edge| matches!(edge.weight(), Edge::Requires)) =>
                {
                    Some(ix)
                }
                _ => None,
            })
            .collect::<Vec<_>>();

        tracing::debug!("Estimating the cost for {} nodes", nodes.len());

        let mut updated_any_node = true;
        let mut iter_count = 0;
        loop {
            if iter_count >= MAX_ITERATIONS {
                tracing::warn!("Exceeded maximum number of cost estimation iterations: {MAX_ITERATIONS}");
                break;
            }
            if !updated_any_node {
                break;
            }

            updated_any_node = false;
            iter_count += 1;

            for &dependent_node in &nodes {
                let mut requirements_cost = self
                    .graph
                    .edges_directed(dependent_node, Direction::Outgoing)
                    .filter_map(|edge| match edge.weight() {
                        Edge::Requires => Some(edge.target()),
                        _ => None,
                    })
                    .flat_map(|required_node| {
                        self.graph
                            .edges_directed(required_node, Direction::Incoming)
                            .filter_map(|edge| match edge.weight() {
                                Edge::Resolves => Some(edge.source()),
                                _ => None,
                            })
                            .filter_map(|resolver_of_required_node| {
                                let ancestor = self.find_common_ancestor(dependent_node, resolver_of_required_node)?;
                                self.estimate_field_cost_from(ancestor, resolver_of_required_node)
                            })
                            .min()
                    })
                    .max()
                    .unwrap_or(0);
                let incoming_edge = self
                    .graph
                    .edges_directed(dependent_node, Direction::Incoming)
                    .next()
                    .expect("Invariant: Resolver always have a single incoming edge")
                    .id();
                match self.graph.edge_weight_mut(incoming_edge).unwrap() {
                    Edge::Resolver(edge_cost) => {
                        // resolver have an inherent cost of 1.
                        requirements_cost += 1;
                        if *edge_cost != requirements_cost {
                            updated_any_node = true;
                            *edge_cost = requirements_cost
                        }
                    }
                    Edge::CanResolveField(edge_cost) => {
                        if *edge_cost != requirements_cost {
                            updated_any_node = true;
                            *edge_cost = requirements_cost
                        }
                    }
                    _ => (),
                }
            }
        }
    }

    fn find_common_ancestor(&self, mut left: NodeIndex, mut right: NodeIndex) -> Option<NodeIndex> {
        let mut ancestors = Vec::new();
        while let Some(l) = self.graph.neighbors_directed(left, Direction::Incoming).next() {
            ancestors.push(l);
            left = l;
        }

        while let Some(r) = self.graph.neighbors_directed(right, Direction::Incoming).next() {
            if ancestors.contains(&r) {
                return Some(r);
            }
            right = r;
        }

        tracing::error!("No common ancestor found? not even root?!");
        None
    }

    fn estimate_field_cost_from(&self, source: NodeIndex, mut target: NodeIndex) -> Option<u16> {
        if source == target {
            return Some(0);
        }

        let mut total_cost = 0;
        while let Some((parent, cost)) = self
            .graph
            .edges_directed(target, Direction::Incoming)
            .filter_map(|edge| match edge.weight() {
                Edge::Resolver(cost) => Some((edge.source(), *cost)),
                Edge::CanResolveField(cost) => Some((edge.source(), *cost)),
                _ => None,
            })
            .next()
        {
            total_cost += cost;
            if parent == source {
                return Some(total_cost);
            }
            target = parent;
        }

        tracing::error!("No path from source to target?");
        None
    }
}
