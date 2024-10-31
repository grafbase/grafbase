use std::num::NonZero;

use fixedbitset::FixedBitSet;
use id_newtypes::IdRange;
use itertools::Itertools;
use petgraph::{
    prelude::StableGraph,
    stable_graph::{EdgeIndex, EdgeReference, NodeIndex},
    visit::{EdgeRef, IntoNodeReferences},
    Direction,
};

use crate::{dot_graph::Attrs, steiner_tree};

use super::{Cost, Edge, Node, Operation, OperationGraph};

/// The solver is responsible for finding the optimal path from the root to the query fields.
/// There are two cores aspects to this, expressing the problem as a Steiner tree problem and
/// solving it with an appropriate algorithm.
///
/// For the first part, the most difficult aspect are dispensable requirements, meaning only needed
/// in certain paths.
/// We don't know whether we'll need them and we don't want to retrieve them if not necessary. To take them
/// into account, we adjust the cost of edges that require them. If requirements can be trivially
/// provided by the parent resolver, no cost is added. If it needs intermediate resolvers not (yet)
/// part of the Steiner it incurs an extra cost.
///
/// As this extra cost changes every time we change the Steiner tree, we have to adjust those while
/// constructing it.
pub(crate) struct Solver<'g, 'ctx, Op: Operation> {
    operation_graph: &'g OperationGraph<'ctx, Op>,
    algorithm: steiner_tree::ShortestPathAlgorithm<&'g StableGraph<Node<Op::FieldId>, Edge>>,
    /// Keeps track of dispensable requirements to adjust edge cost, ideally we'd like to avoid
    /// them.
    dispensable_requirements_metadata: DispensableRequirementsMetadata,
    /// Temporary storage for extra terminals to be added to the algorithm.
    tmp_extra_terminals: Vec<NodeIndex>,
}

pub(crate) struct Solution {
    pub node_bitset: FixedBitSet,
}

impl<'g, 'ctx, Op: Operation> Solver<'g, 'ctx, Op> {
    pub fn initialize(operation_graph: &'g OperationGraph<'ctx, Op>) -> crate::Result<Self> {
        let terminals = operation_graph
            .graph
            .node_references()
            .filter_map(|(node_ix, node)| match node {
                Node::QueryField(field) if field.is_scalar() && field.is_indispensable() => Some(node_ix),
                _ => None,
            });
        let node_filter = |(node_ix, node): (NodeIndex, &Node<Op::FieldId>)| match node {
            Node::Root | Node::Resolver(_) | Node::ProvidableField(_) => Some(node_ix),
            Node::QueryField(field) => {
                if field.is_scalar() {
                    Some(node_ix)
                } else {
                    None
                }
            }
        };
        let edge_filter = |edge: EdgeReference<'_, Edge, _>| match edge.weight() {
            // Resolvers have an inherent cost of 1.
            Edge::CreateChildResolver => Some((edge.id(), edge.source(), edge.target(), 1)),
            Edge::CanProvide | Edge::Provides => Some((edge.id(), edge.source(), edge.target(), 0)),
            Edge::Field | Edge::TypenameField | Edge::HasChildResolver | Edge::Requires => None,
        };

        let algorithm = steiner_tree::ShortestPathAlgorithm::initialize(
            &operation_graph.graph,
            node_filter,
            edge_filter,
            operation_graph.root_ix,
            terminals,
        );

        let mut solver = Self {
            operation_graph,
            algorithm,
            dispensable_requirements_metadata: DispensableRequirementsMetadata::default(),
            tmp_extra_terminals: Vec::new(),
        };

        solver.populate_requirement_metadata()?;
        solver.cost_fixed_point_iteration()?;

        tracing::debug!("Solver populated:\n{}", solver.to_pretty_dot_graph());

        Ok(solver)
    }

    pub fn solve(mut self) -> crate::Result<Solution> {
        self.execute()?;
        Ok(Solution {
            node_bitset: self.algorithm.operation_graph_bitset(),
        })
    }

    /// Solves the Steiner tree problem for the resolvers of our operation graph.
    pub fn execute(&mut self) -> crate::Result<()> {
        loop {
            let has_terminals_left = self.algorithm.continue_steiner_tree_growth();
            let added_new_terminals = self.cost_fixed_point_iteration()?;
            if !has_terminals_left && !added_new_terminals {
                break;
            }
        }
        tracing::debug!("Solver finished:\n{}", self.to_pretty_dot_graph());

        Ok(())
    }

    /// For each node with dispensable requirements, we need its incoming edges cost to reflect
    /// their cost if we were to chose that edge. Those dispensable requirements would then become
    /// indispensable and added to the list of terminals we must find in the Steiner tree.
    ///
    /// A node may have multiple incoming edges being potentially resolved by different resolvers.
    /// This may have implications on the requirements, so we recursively consider any parent incoming edge to
    /// be free as long as there is only one parent. We had to take that path after all. This
    /// allow us to more appropriately reflect cost differences.
    ///
    /// This method populates all the necessary metadata used to compute the extra requirements cost.
    fn populate_requirement_metadata(&mut self) -> crate::Result<()> {
        for (node_ix, node) in self.operation_graph.graph.node_references() {
            if !matches!(node, Node::Resolver(_) | Node::ProvidableField(_)) {
                continue;
            }

            let extra_required_node_ids = self.dispensable_requirements_metadata.extend_extra_required_nodes(
                self.operation_graph
                    .graph
                    .edges_directed(node_ix, Direction::Outgoing)
                    .filter(|edge| {
                        matches!(edge.weight(), Edge::Requires)
                            && self.operation_graph.graph[edge.target()]
                                .as_query_field()
                                .map(|field| !field.is_indispensable() && field.is_scalar())
                                .unwrap_or_default()
                    })
                    .map(|edge| edge.target()),
            );
            if extra_required_node_ids.is_empty() {
                continue;
            }

            for edge in self.operation_graph.graph.edges_directed(node_ix, Direction::Incoming) {
                if !matches!(edge.weight(), Edge::CreateChildResolver | Edge::CanProvide) {
                    continue;
                }

                let mut parent = edge.source();
                // This will at least include the ProvidableField & Resolver that led to the
                // parent. As we'll necessarily take them for this particular edge, they'll be set
                // to 0 cost while estimating the requirement cost.
                let zero_cost_parent_edge_ids =
                    self.dispensable_requirements_metadata
                        .extend_zero_cost_parent_edges(std::iter::from_fn(|| {
                            let mut grand_parents = self
                                .operation_graph
                                .graph
                                .edges_directed(parent, Direction::Incoming)
                                .filter(|edge| matches!(edge.weight(), Edge::CreateChildResolver | Edge::CanProvide));

                            let first = grand_parents.next()?;
                            if grand_parents.next().is_none() {
                                parent = first.source();
                                Some(first.id())
                            } else {
                                None
                            }
                        }));
                self.dispensable_requirements_metadata
                    .edges
                    .push(EdgeWithDispensableRequirements {
                        edge_ix: edge.id(),
                        base_cost: match edge.weight() {
                            Edge::CreateChildResolver => 1,
                            _ => 0,
                        },
                        extra_required_node_ids,
                        zero_cost_parent_edge_ids,
                    });
            }
        }

        Ok(())
    }

    /// Updates the cost of edges based on the requirements of the nodes.
    /// We iterate until cost becomes stable or we exhausted the maximum number of iterations which
    /// likely indicates a requirement cycle.
    fn cost_fixed_point_iteration(&mut self) -> crate::Result<bool> {
        debug_assert!(self.tmp_extra_terminals.is_empty());
        let mut i = 0;
        loop {
            i += 1;
            self.generate_cost_updates_based_on_requirements();
            if !self.algorithm.apply_all_cost_updates()
                || self
                    .dispensable_requirements_metadata
                    .independent_cost
                    .unwrap_or_default()
            {
                break;
            }
            if i > 100 {
                return Err(crate::Error::RequirementCycleDetected);
            }
        }
        // If it's the first time we do the fixed point iteration and we didn't do more than 2
        // iterations (one for updating, one for checking nothing changed). It means there is no
        // dependency between requirements cost. So we can skip it in the next iterations.
        self.dispensable_requirements_metadata
            .independent_cost
            .get_or_insert(i == 2);
        let new_terminals = !self.tmp_extra_terminals.is_empty();
        self.tmp_extra_terminals.sort_unstable();
        self.algorithm
            .extend_terminals(self.tmp_extra_terminals.drain(..).dedup());
        Ok(new_terminals)
    }

    /// For all edges with dispensable requirements, we estimate the cost of the extra requirements
    /// by computing cost of adding them to the current Steiner tree plus the base cost of the
    /// edge.
    fn generate_cost_updates_based_on_requirements(&mut self) {
        let mut i = 0;
        while let Some(EdgeWithDispensableRequirements {
            edge_ix,
            base_cost,
            extra_required_node_ids,
            zero_cost_parent_edge_ids,
        }) = self.dispensable_requirements_metadata.edges.get(i)
        {
            let (source_ix, target_ix) = self.operation_graph.graph.edge_endpoints(*edge_ix).unwrap();
            if self.algorithm.contains_node(target_ix) {
                self.tmp_extra_terminals.extend(
                    self.dispensable_requirements_metadata[*extra_required_node_ids]
                        .iter()
                        .copied(),
                );
                self.dispensable_requirements_metadata.edges.swap_remove(i);
                continue;
            }

            let new_cost = base_cost
                + self.algorithm.estimate_extra_cost(
                    self.dispensable_requirements_metadata[*zero_cost_parent_edge_ids]
                        .iter()
                        .copied(),
                    self.dispensable_requirements_metadata[*extra_required_node_ids]
                        .iter()
                        .copied(),
                );

            self.algorithm.insert_edge_cost_update(source_ix, *edge_ix, new_cost);

            i += 1;
        }
    }

    pub fn to_pretty_dot_graph(&self) -> String {
        self.algorithm.to_dot_graph(
            |cost, is_in_steiner_tree| {
                Attrs::label_if(cost > 0, cost.to_string())
                    .bold()
                    .with_if(is_in_steiner_tree, "color=forestgreen,fontcolor=forestgreen")
                    .with_if(!is_in_steiner_tree, "color=royalblue,fontcolor=royalblue,style=dashed")
                    .to_string()
            },
            |node_id, is_in_steiner_tree| {
                self.operation_graph
                    .graph
                    .node_weight(node_id)
                    .unwrap()
                    .pretty_label(self.operation_graph)
                    .with_if(!is_in_steiner_tree, "style=dashed")
                    .with_if(is_in_steiner_tree, "color=forestgreen")
                    .to_string()
            },
        )
    }

    /// Use https://dreampuf.github.io/GraphvizOnline
    /// or `echo '..." | dot -Tsvg` from graphviz
    #[cfg(test)]
    pub fn to_dot_graph(&self) -> String {
        self.algorithm.to_dot_graph(
            |cost, is_in_steiner_tree| format!("cost={cost}, steiner={}", is_in_steiner_tree as usize),
            |node_id, is_in_steiner_tree| {
                Attrs::label(
                    self.operation_graph
                        .graph
                        .node_weight(node_id)
                        .unwrap()
                        .label(self.operation_graph),
                )
                .with(format!("steiner={}", is_in_steiner_tree as usize))
                .to_string()
            },
        )
    }
}

impl<'g, 'ctx, Op: Operation> std::fmt::Debug for Solver<'g, 'ctx, Op> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Solver").finish_non_exhaustive()
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, id_derives::Id)]
struct ExtraRequiredNodeId(NonZero<u32>);

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, id_derives::Id)]
struct ZeroCostParentEdgeId(NonZero<u32>);

#[derive(Default, id_derives::IndexedFields)]
struct DispensableRequirementsMetadata {
    edges: Vec<EdgeWithDispensableRequirements>,
    #[indexed_by(ExtraRequiredNodeId)]
    extra_required_nodes: Vec<NodeIndex>,
    #[indexed_by(ZeroCostParentEdgeId)]
    zero_cost_parent_edges: Vec<EdgeIndex>,
    independent_cost: Option<bool>,
}

#[derive(Clone, Copy)]
struct EdgeWithDispensableRequirements {
    edge_ix: EdgeIndex,
    base_cost: Cost,
    extra_required_node_ids: IdRange<ExtraRequiredNodeId>,
    zero_cost_parent_edge_ids: IdRange<ZeroCostParentEdgeId>,
}

impl DispensableRequirementsMetadata {
    fn extend_extra_required_nodes(
        &mut self,
        nodes: impl IntoIterator<Item = NodeIndex>,
    ) -> IdRange<ExtraRequiredNodeId> {
        let start = self.extra_required_nodes.len();
        self.extra_required_nodes.extend(nodes);
        IdRange::from(start..self.extra_required_nodes.len())
    }

    fn extend_zero_cost_parent_edges(
        &mut self,
        edges: impl IntoIterator<Item = EdgeIndex>,
    ) -> IdRange<ZeroCostParentEdgeId> {
        let start = self.zero_cost_parent_edges.len();
        self.zero_cost_parent_edges.extend(edges);
        IdRange::from(start..self.zero_cost_parent_edges.len())
    }
}
