use std::ops::ControlFlow;

use itertools::Itertools as _;
use petgraph::graph::{EdgeIndex, NodeIndex};

use crate::{
    Cost, SolutionSpaceGraph,
    solve::{
        context::{SteinerContext, SteinerGraph},
        requirements::{DispensableRequirements, DispensableRequirementsMetadata},
        steiner_tree::{GreedyFlac, SteinerTree},
    },
};

pub(crate) struct CostUpdaterState {
    independent_requirements: Option<bool>,
    /// Temporary storage for extra terminals to be added to the algorithm.
    tmp_extra_terminals: Vec<NodeIndex>,
    tmp_steiner_tree: SteinerTree,
    tmp_flac: GreedyFlac,
    has_updated_cost: bool,
}

impl CostUpdaterState {
    pub(crate) fn new(ctx: &SteinerContext<&SolutionSpaceGraph<'_>, SteinerGraph>) -> Self {
        Self {
            independent_requirements: None,
            tmp_extra_terminals: Vec::new(),
            tmp_steiner_tree: SteinerTree::new(&ctx.graph, ctx.root_ix),
            tmp_flac: GreedyFlac::new(&ctx.graph, Vec::new()),
            has_updated_cost: false,
        }
    }
}

pub(crate) struct CostUpdater<'a, 'q, 'schema> {
    pub state: &'a mut CostUpdaterState,
    pub flac: &'a mut GreedyFlac,
    pub steiner_tree: &'a SteinerTree,
    pub dispensable_requirements_metadata: &'a mut DispensableRequirementsMetadata,
    pub ctx: &'a mut SteinerContext<&'q SolutionSpaceGraph<'schema>, SteinerGraph>,
}

impl std::ops::Deref for CostUpdater<'_, '_, '_> {
    type Target = CostUpdaterState;
    fn deref(&self) -> &Self::Target {
        self.state
    }
}

impl std::ops::DerefMut for CostUpdater<'_, '_, '_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.state
    }
}

impl CostUpdater<'_, '_, '_> {
    /// Updates the cost of edges based on the requirements of the nodes.
    /// We iterate until cost becomes stable or we exhausted the maximum number of iterations which
    /// likely indicates a requirement cycle.
    pub fn cost_fixed_point_iteration(&mut self) -> crate::Result<ControlFlow<()>> {
        debug_assert!(self.tmp_extra_terminals.is_empty());
        let mut i = 0;
        loop {
            i += 1;
            self.generate_cost_updates_based_on_requirements();
            let has_updates = std::mem::take(&mut self.has_updated_cost);
            if !has_updates || self.independent_requirements.unwrap_or_default() {
                break;
            }
            if i > 100 {
                return Err(crate::Error::RequirementCycleDetected);
            }
        }
        // If it's the first time we do the fixed point iteration and we didn't do more than 2
        // iterations (one for updating, one for checking nothing changed). It means there is no
        // dependency between requirements cost. So we can skip it in the next iterations.
        self.independent_requirements.get_or_insert(i == 2);
        let new_terminals = !self.tmp_extra_terminals.is_empty();
        self.tmp_extra_terminals.sort_unstable();
        self.flac.extend_terminals(
            self.state
                .tmp_extra_terminals
                .drain(..)
                .dedup()
                .map(|node| self.ctx.to_node_ix(node))
                .filter(|idx| !self.steiner_tree.nodes[idx.index()]),
        );

        Ok(if new_terminals {
            ControlFlow::Continue(())
        } else {
            ControlFlow::Break(())
        })
    }

    /// For all edges with dispensable requirements, we estimate the cost of the extra requirements
    /// by computing cost of adding them to the current Steiner tree plus the base cost of the
    /// edge.
    fn generate_cost_updates_based_on_requirements(&mut self) {
        let mut i = 0;
        while let Some((node_id, extra_required_node_ids)) =
            self.dispensable_requirements_metadata.free_requirements.get(i).copied()
        {
            if self.steiner_tree.nodes[self.ctx.to_node_ix(node_id).index()] {
                self.state.tmp_extra_terminals.extend(
                    self.dispensable_requirements_metadata[extra_required_node_ids]
                        .iter()
                        .copied(),
                );
                self.dispensable_requirements_metadata.free_requirements.swap_remove(i);
            } else {
                i += 1;
            }
        }

        i = 0;
        while let Some(DispensableRequirements {
            extra_required_node_ids,
            unavoidable_parent_edge_ids,
            incoming_edge_and_cost_ids,
        }) = self
            .dispensable_requirements_metadata
            .maybe_costly_requirements
            .get(i)
            .copied()
        {
            if self.dispensable_requirements_metadata[incoming_edge_and_cost_ids]
                .iter()
                .any(|(incoming_edge, _)| {
                    let (_, target_ix) = self.ctx.query_graph.edge_endpoints(*incoming_edge).unwrap();
                    self.steiner_tree.nodes[self.ctx.to_node_ix(target_ix).index()]
                })
            {
                self.state.tmp_extra_terminals.extend(
                    self.dispensable_requirements_metadata[extra_required_node_ids]
                        .iter()
                        .copied(),
                );
                self.dispensable_requirements_metadata
                    .maybe_costly_requirements
                    .swap_remove(i);
                continue;
            }

            let unavoidable_edges = self.dispensable_requirements_metadata[unavoidable_parent_edge_ids].to_vec();
            let extra_terminals = self.dispensable_requirements_metadata[extra_required_node_ids].to_vec();
            let extra_cost = self.estimate_extra_cost(&unavoidable_edges, &extra_terminals);

            let edges_and_costs = self.dispensable_requirements_metadata[incoming_edge_and_cost_ids].to_vec();
            for (incoming_edge, cost) in edges_and_costs {
                let (source_ix, _) = self.ctx.query_graph.edge_endpoints(incoming_edge).unwrap();
                self.insert_edge_cost_update(source_ix, incoming_edge, cost + extra_cost);
            }

            i += 1;
        }
    }

    fn estimate_extra_cost(&mut self, steiner_tree_edges: &[EdgeIndex], extra_terminals: &[NodeIndex]) -> Cost {
        self.tmp_flac.reset();
        self.state.tmp_steiner_tree.nodes.clone_from(&self.steiner_tree.nodes);
        self.state.tmp_steiner_tree.edges.clone_from(&self.steiner_tree.edges);
        self.tmp_steiner_tree.total_weight = 0;

        for edge_id in steiner_tree_edges {
            let edge_ix = self.ctx.to_edge_ix(*edge_id);
            self.tmp_steiner_tree.edges.insert(edge_ix.index());
            let (_, dst) = self.ctx.graph.edge_endpoints(edge_ix).unwrap();
            self.tmp_steiner_tree.nodes.insert(dst.index());
        }

        self.state
            .tmp_flac
            .extend_terminals(extra_terminals.iter().map(|node| self.ctx.to_node_ix(*node)));
        self.state
            .tmp_flac
            .run(&self.ctx.graph, &mut self.state.tmp_steiner_tree);

        self.tmp_steiner_tree.total_weight
    }

    fn insert_edge_cost_update(&mut self, _source_id: NodeIndex, edge_id: EdgeIndex, cost: Cost) {
        let edge_ix = self.ctx.to_edge_ix(edge_id);
        let old = std::mem::replace(&mut self.ctx.graph[edge_ix], cost);
        self.has_updated_cost |= old != cost;
    }
}
