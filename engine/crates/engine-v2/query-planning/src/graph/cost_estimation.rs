use id_newtypes::IdRange;
use itertools::Itertools;
use petgraph::{
    graph::{EdgeIndex, NodeIndex},
    prelude::StableGraph,
    visit::{EdgeRef, IntoEdgeReferences, IntoNodeReferences},
    Direction,
};
use std::num::NonZero;

use crate::{Edge, Node, Operation};

use super::builder::OperationGraphBuilder;

const MAX_COST_ITERATIONS: usize = 20;

pub type Cost = u16;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub(crate) struct EdgeCostId(NonZero<u32>);

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub(crate) struct RequirementId(NonZero<u16>);

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub(crate) struct AncestorId(NonZero<u32>);

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub(crate) struct OriginRootAncestorId(NonZero<u32>);

#[derive(id_derives::IndexedFields)]
pub(crate) struct CostEstimator {
    // Ordered by EdgeIndex
    edge_to_impacted_requirement: Vec<(EdgeIndex, RequirementId)>,
    #[indexed_by(EdgeCostId)]
    edge_cost: Vec<Cost>,
    #[indexed_by(RequirementId)]
    requirements: Vec<Requirement>,
    #[indexed_by(AncestorId)]
    all_ancestors: Vec<Ancestor>,
    #[indexed_by(OriginRootAncestorId)]
    all_origin_root_ancestors: Vec<OriginRootAncestor>,
}

pub(crate) struct OriginRootAncestor {
    origin_query_field_ix: NodeIndex,
    ancestor_id: AncestorId,
}

pub(crate) struct Requirement {
    query_field_ix: NodeIndex,
    // Ordering by NodeIndex
    origin_root_ancestor_ids: IdRange<OriginRootAncestorId>,
    // In topological order
    ancestry_ids: IdRange<AncestorId>,
}

pub(crate) struct Ancestor {
    parent: AncestorId,
    edge_cost_id: EdgeCostId,
    cumulative_cost_to_reach_requirement: Cost,
}

impl<'ctx, Op: Operation> OperationGraphBuilder<'ctx, Op> {
    #[allow(unused)]
    pub(crate) fn build_cost_estimator(&mut self) -> crate::Result<CostEstimator> {
        let mut estimator = CostEstimator {
            edge_cost: vec![0; self.edge_cost_count() + 1],
            edge_to_impacted_requirement: Vec::new(),
            requirements: Vec::new(),
            all_ancestors: Vec::new(),
            all_origin_root_ancestors: Vec::new(),
        };

        estimator.populate_from(&mut self.graph);
        estimator.fixed_point_iteration(
            &self.graph,
            (0..estimator.requirements.len()).map(RequirementId::from).collect(),
        )?;

        Ok(estimator)
    }
}

impl CostEstimator {
    // for each incoming edge on a node with requirements:
    // - compute ancestors we need to keep track off for _all_ requirements.
    // - can't really merge requirements with parent as individual fields have a specific cost or
    // may be coming from entity join.
    // - remove any requirement with a parent edge/node already in it. Cumulate all edgecostid?
    // kkproblem with edge cost id, union find actually
    // - remove any ancestor with parent already in it. It means there is a requirement that is a
    // parent of us, so
    // - end up with a list of (ancestor, requirement id)
    fn populate_from<Id>(&mut self, graph: &mut StableGraph<Node<Id>, Edge>) {
        let mut stack = Vec::new();
        let mut requirement_origins = Vec::new();
        let mut origin_outgoing_edge_cost_ids = Vec::new();
        let mut ancestry_edges = Vec::new();

        // Initialize edge cost
        for edge in graph.edge_references() {
            // Creating a new resolver has an inherent cost of 1
            if let Edge::CreateChildResolver { id } = edge.weight() {
                self[*id] = 1;
            }
        }
        let zero_edge_cost_id = EdgeCostId::from(self.edge_cost.len() - 1);

        let zero_cost_ancestor_id = AncestorId::from(0usize);
        self.all_ancestors.push(Ancestor {
            // cycle with itself, but it's never part of any ancestry, so it's always only
            // referenced as a parent.
            parent: zero_cost_ancestor_id,
            edge_cost_id: zero_edge_cost_id,
            cumulative_cost_to_reach_requirement: 0,
        });

        for (query_field_ix, node) in graph.node_references() {
            if !matches!(node, Node::QueryField(_)) {
                continue;
            }
            debug_assert!(requirement_origins.is_empty());
            requirement_origins.extend(
                graph
                    .edges_directed(query_field_ix, Direction::Incoming)
                    .filter_map(|edge| match edge.weight() {
                        Edge::Requires { origin_query_field_ix } => Some(*origin_query_field_ix),
                        _ => None,
                    }),
            );

            let Some(min_origin_depth) = requirement_origins
                .iter()
                .map(|origin_query_field_ix| graph[*origin_query_field_ix].query_depth())
                .min()
            else {
                continue;
            };

            // A resolver query depth is the depth of the field it resolves. So any resolvers
            // between the origin and the required field has necessarily a lower depth as they
            // resolves nested fields of the origin.
            let min_resolver_depth = min_origin_depth + 1;

            let ancestry_start = self.all_ancestors.len();
            debug_assert!(stack.is_empty());
            stack.extend(graph.edges_directed(query_field_ix, Direction::Incoming).filter_map(
                |edge| match edge.weight() {
                    Edge::Provides => Some((zero_cost_ancestor_id, 0, edge.source())),
                    _ => None,
                },
            ));
            debug_assert!(ancestry_edges.is_empty());
            while let Some((parent_ancestor_id, parent_cost, node_ix)) = stack.pop() {
                for edge in graph.edges_directed(node_ix, Direction::Incoming) {
                    match edge.weight() {
                        Edge::CreateChildResolver { id } | Edge::CanProvide { id } => {
                            let cost = self[*id] + parent_cost;
                            let ancestor_id = AncestorId::from(self.all_ancestors.len());
                            self.all_ancestors.push(Ancestor {
                                parent: parent_ancestor_id,
                                edge_cost_id: *id,
                                cumulative_cost_to_reach_requirement: cost,
                            });
                            ancestry_edges.push(edge.id());

                            let query_field_ix = edge.source();
                            if graph[query_field_ix].query_depth() >= min_resolver_depth {
                                stack.push((ancestor_id, cost, query_field_ix));
                            }
                        }
                        Edge::Provides
                        | Edge::Field
                        | Edge::TypenameField
                        | Edge::HasChildResolver
                        | Edge::Requires { .. } => continue,
                    };
                }
            }
            let ancestry_end = self.all_ancestors.len();

            let origin_root_ancestors_start = self.all_origin_root_ancestors.len();
            requirement_origins.sort_unstable();
            for origin_query_field_ix in requirement_origins.drain(..) {
                origin_outgoing_edge_cost_ids.clear();
                origin_outgoing_edge_cost_ids.extend(
                    graph
                        .edges_directed(origin_query_field_ix, Direction::Outgoing)
                        .filter_map(|edge| match edge.weight() {
                            Edge::CreateChildResolver { id } | Edge::CanProvide { id } => Some(*id),
                            Edge::Field
                            | Edge::TypenameField
                            | Edge::HasChildResolver
                            | Edge::Provides
                            | Edge::Requires { .. } => None,
                        }),
                );
                for (i, ancestor) in self.all_ancestors[ancestry_start..ancestry_end].iter().enumerate() {
                    if origin_outgoing_edge_cost_ids.contains(&ancestor.edge_cost_id) {
                        self.all_origin_root_ancestors.push(OriginRootAncestor {
                            origin_query_field_ix,
                            ancestor_id: AncestorId::from(ancestry_start + i),
                        });
                    }
                }
            }
            let origin_root_ancestors_end = self.all_origin_root_ancestors.len();

            let requirement_id = RequirementId::from(self.requirements.len());
            self.requirements.push(Requirement {
                query_field_ix,
                ancestry_ids: IdRange::from(ancestry_start..ancestry_end),
                origin_root_ancestor_ids: IdRange::from(origin_root_ancestors_start..origin_root_ancestors_end),
            });
            ancestry_edges.sort_unstable();
            self.edge_to_impacted_requirement
                .extend(ancestry_edges.drain(..).dedup().map(|edge| (edge, requirement_id)));
        }

        for (i, requirement) in self.requirements.iter().enumerate() {
            let id = RequirementId::from(i);
            graph[requirement.query_field_ix]
                .as_query_field_mut()
                .unwrap()
                .requirement_id = Some(id);
        }

        self.edge_to_impacted_requirement.sort_unstable();
    }

    // Edge cost should be converge to a fixed point as long as there isn't any requirement cycle.
    // To speed up the calculation we keep track of the impacted requirements of the previous
    // round. We start with all the requirements initially as they've all have been just created.
    fn fixed_point_iteration<Id>(
        &mut self,
        graph: &StableGraph<Node<Id>, Edge>,
        mut impacted_requirements: Vec<RequirementId>,
    ) -> crate::Result<()> {
        let mut iterations = 0;
        let mut impacted_nodes = impacted_requirements
            .drain(..)
            .flat_map(|requirement_id| {
                graph
                    .edges_directed(self[requirement_id].query_field_ix, Direction::Incoming)
                    .filter_map(|edge| match edge.weight() {
                        Edge::Requires { .. } => match graph[edge.source()] {
                            Node::Resolver(_) | Node::ProvidableField(_) => Some(edge.source()),
                            Node::QueryField(_) | Node::Root => None,
                        },
                        _ => None,
                    })
            })
            .collect::<Vec<_>>();

        while !impacted_nodes.is_empty() {
            debug_assert!(impacted_requirements.is_empty());
            impacted_nodes.sort_unstable();
            for node_ix in impacted_nodes.drain(..).dedup() {
                // For now the cost of each incoming edge is assumed to be equal. Later we should
                // take into account from which subgraph they come.
                let cost = graph
                    .edges_directed(node_ix, Direction::Outgoing)
                    .filter_map(|edge| match edge.weight() {
                        Edge::Requires { origin_query_field_ix } => Some((
                            *origin_query_field_ix,
                            graph[edge.target()].as_query_field().unwrap().requirement_id.unwrap(),
                        )),
                        _ => None,
                    })
                    .map(|(origin_query_field_ix, requirement_id)| {
                        self.requirement_cost(origin_query_field_ix, requirement_id)
                    })
                    .max()
                    .unwrap_or_default();
                for edge in graph.edges_directed(node_ix, Direction::Incoming) {
                    let (id, cost) = match edge.weight() {
                        Edge::CreateChildResolver { id } => (id, cost + 1),
                        Edge::CanProvide { id } => (id, cost),
                        _ => continue,
                    };
                    if self[*id] != cost {
                        impacted_requirements.extend(self.impacted_requirements_for(edge.id()));
                        self[*id] = cost;
                    }
                }
            }

            debug_assert!(impacted_nodes.is_empty());
            impacted_requirements.sort_unstable();
            for requirement_id in impacted_requirements.drain(..).dedup() {
                // Updating all paths to the requirement. As we iterate over ancestors in
                // topological order, we're sure to update them in order.
                for ancestor_id in self[requirement_id].ancestry_ids {
                    let ancestor = &self[ancestor_id];
                    let cost = self[ancestor.edge_cost_id] + self[ancestor.parent].cumulative_cost_to_reach_requirement;
                    self[ancestor_id].cumulative_cost_to_reach_requirement = cost;
                }
                impacted_nodes.extend(
                    graph
                        .edges_directed(self[requirement_id].query_field_ix, Direction::Incoming)
                        .filter_map(|edge| match edge.weight() {
                            Edge::Requires { .. } => match graph[edge.source()] {
                                Node::Resolver(_) | Node::ProvidableField(_) => Some(edge.source()),
                                Node::QueryField(_) | Node::Root => None,
                            },
                            _ => None,
                        }),
                );
            }

            iterations += 1;
            if iterations == MAX_COST_ITERATIONS {
                return Err(crate::Error::RequirementCycleDetected);
            }
        }

        Ok(())
    }

    fn requirement_cost(&self, origin_query_field_ix: NodeIndex, requirement_id: RequirementId) -> Cost {
        self.requirement_ancestor_from_origin_query_field(origin_query_field_ix, requirement_id)
            .map(|ancestor_id| self[ancestor_id].cumulative_cost_to_reach_requirement)
            .min()
            .unwrap_or_default()
    }

    fn requirement_ancestor_from_origin_query_field(
        &self,
        origin_query_field_ix: NodeIndex,
        requirement_id: RequirementId,
    ) -> impl Iterator<Item = AncestorId> + '_ {
        let origin_root_ancestors = &self[self[requirement_id].origin_root_ancestor_ids];
        let offset = origin_root_ancestors.partition_point(|probe| probe.origin_query_field_ix < origin_query_field_ix);
        origin_root_ancestors[offset..]
            .iter()
            .take_while(move |origin_root_ancestor| origin_root_ancestor.origin_query_field_ix == origin_query_field_ix)
            .map(|origin_root_ancestor| origin_root_ancestor.ancestor_id)
    }

    fn impacted_requirements_for(&self, edge_ix: EdgeIndex) -> impl Iterator<Item = RequirementId> + '_ {
        let offset = self
            .edge_to_impacted_requirement
            .partition_point(|(ix, _)| *ix < edge_ix);
        self.edge_to_impacted_requirement[offset..]
            .iter()
            .take_while(move |(ix, _)| *ix == edge_ix)
            .map(|(_, id)| *id)
    }
}
