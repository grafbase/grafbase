use id_newtypes::IdRange;
use petgraph::graph::NodeIndex;

use crate::{
    Cost, SolutionSpaceGraph,
    solve::{
        context::{SteinerContext, SteinerGraph},
        requirements::{
            DispensableRequirements, RequirementsGroup,
            metadata::{RequiredNodeId, UnavoidableParentEdgeId},
        },
        steiner_tree::{GreedyFlac, SteinerTree},
    },
};

pub(crate) struct RequirementAndCostUpdater {
    /// Keeps track of dispensable requirements to adjust edge cost, ideally we'd like to avoid
    /// them.
    dispensable_requirements: DispensableRequirements,
    independent_requirements: Option<bool>,
    /// Temporary storage for extra terminals to be added to the algorithm.
    tmp_extra_terminals: Vec<NodeIndex>,
    tmp_steiner_tree: SteinerTree,
    tmp_flac: GreedyFlac,
    has_updated_cost: bool,
}

impl RequirementAndCostUpdater {
    pub fn new(ctx: &SteinerContext<&SolutionSpaceGraph<'_>, SteinerGraph>) -> crate::Result<Self> {
        let mut dispensable_requirements_metadata = DispensableRequirements::default();
        dispensable_requirements_metadata.ingest(ctx)?;
        Ok(Self {
            dispensable_requirements: dispensable_requirements_metadata,
            independent_requirements: None,
            tmp_extra_terminals: Vec::new(),
            tmp_steiner_tree: SteinerTree::new(&ctx.graph, ctx.root_ix),
            tmp_flac: GreedyFlac::new(&ctx.graph, Vec::new()),
            has_updated_cost: false,
        })
    }

    pub fn run_fixed_point_cost<'s>(
        &'s mut self,
        graph: &mut SteinerGraph,
        steiner_tree: &SteinerTree,
    ) -> crate::Result<&'s mut Vec<NodeIndex>> {
        FixedPointCostAlgorithm {
            state: self,
            steiner_tree,
            graph,
        }
        .run()
    }
}

pub(crate) struct FixedPointCostAlgorithm<'s, 't, 'g> {
    pub state: &'s mut RequirementAndCostUpdater,
    pub steiner_tree: &'t SteinerTree,
    pub graph: &'g mut SteinerGraph,
}

impl std::ops::Deref for FixedPointCostAlgorithm<'_, '_, '_> {
    type Target = RequirementAndCostUpdater;
    fn deref(&self) -> &Self::Target {
        self.state
    }
}

impl std::ops::DerefMut for FixedPointCostAlgorithm<'_, '_, '_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.state
    }
}

impl<'state> FixedPointCostAlgorithm<'state, '_, '_> {
    /// Updates the cost of edges based on the requirements of the nodes.
    /// We iterate until cost becomes stable or we exhausted the maximum number of iterations which
    /// likely indicates a requirement cycle.
    pub fn run(mut self) -> crate::Result<&'state mut Vec<NodeIndex>> {
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
        Ok(&mut self.state.tmp_extra_terminals)
    }

    /// For all edges with dispensable requirements, we estimate the cost of the extra requirements
    /// by computing cost of adding them to the current Steiner tree plus the base cost of the
    /// edge.
    fn generate_cost_updates_based_on_requirements(&mut self) {
        let mut i = 0;
        while let Some((node_id, extra_required_node_ids)) =
            self.dispensable_requirements.free_requirements.get(i).copied()
        {
            if self.steiner_tree[node_id] {
                self.state.tmp_extra_terminals.extend(
                    self.state.dispensable_requirements[extra_required_node_ids]
                        .iter()
                        .copied(),
                );
                self.dispensable_requirements.free_requirements.swap_remove(i);
            } else {
                i += 1;
            }
        }

        i = 0;
        while let Some(RequirementsGroup {
            required_node_ids,
            unavoidable_parent_edge_ids,
            dependent_edge_with_inherent_cost_ids,
        }) = self.dispensable_requirements.groups.get(i).copied()
        {
            if self.dispensable_requirements[dependent_edge_with_inherent_cost_ids]
                .iter()
                .any(|(edge_id, _)| self.steiner_tree[*edge_id])
            {
                // for &(edge_id, cost) in &self.state.dispensable_requirements[dependent_edge_with_inherent_cost_ids] {
                //     let old = std::mem::replace(&mut self.graph[edge_id], cost);
                //     self.state.has_updated_cost |= old != cost;
                // }
                self.state
                    .tmp_extra_terminals
                    .extend(self.state.dispensable_requirements[required_node_ids].iter().copied());
                self.dispensable_requirements.groups.swap_remove(i);
                continue;
            }

            let extra_cost = self.state.estimate_extra_cost(
                self.graph,
                self.steiner_tree,
                unavoidable_parent_edge_ids,
                required_node_ids,
            );

            let edges_and_costs = self.dispensable_requirements[dependent_edge_with_inherent_cost_ids].to_vec();
            for (edge_id, cost) in edges_and_costs {
                let cost = cost + extra_cost;
                let old = std::mem::replace(&mut self.graph[edge_id], cost);
                self.has_updated_cost |= old != cost;
            }

            i += 1;
        }
    }
}

impl RequirementAndCostUpdater {
    fn estimate_extra_cost(
        &mut self,
        graph: &SteinerGraph,
        steiner_tree: &SteinerTree,
        steiner_tree_edges: IdRange<UnavoidableParentEdgeId>,
        extra_terminals: IdRange<RequiredNodeId>,
    ) -> Cost {
        self.tmp_flac.reset();
        self.tmp_steiner_tree.nodes.clone_from(&steiner_tree.nodes);
        self.tmp_steiner_tree.edges.clone_from(&steiner_tree.edges);
        self.tmp_steiner_tree.total_weight = 0;

        for &edge_id in &self.dispensable_requirements[steiner_tree_edges] {
            self.tmp_steiner_tree.edges.insert(edge_id.index());
            let (_, dst) = graph.edge_endpoints(edge_id).unwrap();
            self.tmp_steiner_tree.nodes.insert(dst.index());
        }

        self.tmp_flac
            .extend_terminals(self.dispensable_requirements[extra_terminals].iter().copied());
        self.tmp_flac.run(graph, &mut self.tmp_steiner_tree);

        self.tmp_steiner_tree.total_weight
    }
}
