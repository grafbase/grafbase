use id_newtypes::IdRange;
use petgraph::graph::NodeIndex;

use crate::{
    Cost,
    solve::{
        input::{RequiredSteinerNodeId, RequirementsGroup, SteinerInput, UnavoidableParentSteinerEdgeId},
        steiner_tree::{GreedyFlac, SteinerTree},
    },
};

pub(crate) struct RequirementAndCostUpdater {
    /// Keeps track of dispensable requirements to adjust edge cost, ideally we'd like to avoid
    /// them.
    independent_requirements: Option<bool>,
    /// Temporary storage for extra terminals to be added to the algorithm.
    tmp_extra_terminals: Vec<NodeIndex>,
    tmp_steiner_tree: SteinerTree,
    tmp_flac: GreedyFlac,
}

impl RequirementAndCostUpdater {
    pub fn new(input: &SteinerInput<'_>) -> crate::Result<Self> {
        Ok(Self {
            independent_requirements: None,
            tmp_extra_terminals: Vec::new(),
            tmp_steiner_tree: SteinerTree::new(&input.graph, input.root_node_id),
            tmp_flac: GreedyFlac::new(&input.graph, Vec::new()),
        })
    }

    pub fn run_fixed_point_cost<'s>(
        &'s mut self,
        input: &mut SteinerInput<'_>,
        steiner_tree: &SteinerTree,
    ) -> crate::Result<Update<'s>> {
        FixedPointCostAlgorithm {
            state: self,
            steiner_tree,
            input,
            has_updated_cost: false,
        }
        .run()
    }
}

pub(crate) struct FixedPointCostAlgorithm<'s, 't, 'i, 'schema> {
    pub state: &'s mut RequirementAndCostUpdater,
    pub steiner_tree: &'t SteinerTree,
    pub input: &'i mut SteinerInput<'schema>,
    has_updated_cost: bool,
}

impl std::ops::Deref for FixedPointCostAlgorithm<'_, '_, '_, '_> {
    type Target = RequirementAndCostUpdater;
    fn deref(&self) -> &Self::Target {
        self.state
    }
}

impl std::ops::DerefMut for FixedPointCostAlgorithm<'_, '_, '_, '_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.state
    }
}

pub(crate) struct Update<'a> {
    pub new_terminals: &'a mut Vec<NodeIndex>,
    #[allow(unused)]
    pub has_updated_cost: bool,
}

impl<'state> FixedPointCostAlgorithm<'state, '_, '_, '_> {
    /// Updates the cost of edges based on the requirements of the nodes.
    /// We iterate until cost becomes stable or we exhausted the maximum number of iterations which
    /// likely indicates a requirement cycle.
    pub fn run(mut self) -> crate::Result<Update<'state>> {
        debug_assert!(self.tmp_extra_terminals.is_empty());
        let mut has_updated_cost = false;
        let mut i = 0;
        loop {
            i += 1;
            self.generate_cost_updates_based_on_requirements();
            let has_updated_cost_this_iteration = std::mem::take(&mut self.has_updated_cost);
            has_updated_cost |= has_updated_cost_this_iteration;
            if !has_updated_cost_this_iteration || self.independent_requirements.unwrap_or_default() {
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
        Ok(Update {
            new_terminals: &mut self.state.tmp_extra_terminals,
            has_updated_cost,
        })
    }

    /// For all edges with dispensable requirements, we estimate the cost of the extra requirements
    /// by computing cost of adding them to the current Steiner tree plus the base cost of the
    /// edge.
    fn generate_cost_updates_based_on_requirements(&mut self) {
        let mut i = 0;
        while let Some((node_id, extra_required_node_ids)) = self.input.requirements.free.get(i).copied() {
            if self.steiner_tree[node_id] {
                self.state.tmp_extra_terminals.extend(
                    self.input.requirements[extra_required_node_ids]
                        .iter()
                        .copied()
                        .filter(|&n| !self.steiner_tree[n]),
                );
                self.input.requirements.free.swap_remove(i);
            } else {
                i += 1;
            }
        }

        i = 0;
        while let Some(RequirementsGroup {
            required_node_ids,
            unavoidable_parent_edge_ids,
            dependent_edge_with_inherent_cost_ids,
        }) = self.input.requirements.groups.get(i).copied()
        {
            if self.input.requirements[dependent_edge_with_inherent_cost_ids]
                .iter()
                .any(|(edge_id, _)| self.steiner_tree[*edge_id])
            {
                // One of the dependent edge is part of the steiner tree, so the requirements
                // aren't optional anymore. Every edge that isn't part of the steiner tree is set back
                // to its inherent cost.
                for &(edge_id, inherent_cost) in &self.input.requirements[dependent_edge_with_inherent_cost_ids] {
                    if !self.steiner_tree[edge_id] {
                        let old = std::mem::replace(&mut self.input.graph[edge_id], inherent_cost);
                        self.has_updated_cost |= old != inherent_cost;
                    }
                }
                self.state.tmp_extra_terminals.extend(
                    self.input.requirements[required_node_ids]
                        .iter()
                        .copied()
                        .filter(|&n| !self.steiner_tree[n]),
                );
                self.input.requirements.groups.swap_remove(i);
                continue;
            }

            let requirements_cost = self.state.estimate_requirements_cost(
                self.input,
                self.steiner_tree,
                unavoidable_parent_edge_ids,
                required_node_ids,
            );

            let edges_and_costs = self.input.requirements[dependent_edge_with_inherent_cost_ids].to_vec();
            for (edge_id, inherent_cost) in edges_and_costs {
                let cost = inherent_cost + requirements_cost;
                let old = std::mem::replace(&mut self.input.graph[edge_id], cost);
                self.has_updated_cost |= old != cost;
            }

            i += 1;
        }
    }
}

impl RequirementAndCostUpdater {
    fn estimate_requirements_cost(
        &mut self,
        input: &mut SteinerInput<'_>,
        steiner_tree: &SteinerTree,
        unavoidable_parent_edge_ids: IdRange<UnavoidableParentSteinerEdgeId>,
        required_node_ids: IdRange<RequiredSteinerNodeId>,
    ) -> Cost {
        self.tmp_flac.reset_terminals();
        self.tmp_steiner_tree.nodes.clone_from(&steiner_tree.nodes);
        self.tmp_steiner_tree.edges.clone_from(&steiner_tree.edges);
        self.tmp_steiner_tree.total_weight = 0;

        for &edge_id in &input.requirements[unavoidable_parent_edge_ids] {
            self.tmp_steiner_tree.edges.insert(edge_id.index());
            let (_, dst) = input.graph.edge_endpoints(edge_id).unwrap();
            self.tmp_steiner_tree.nodes.insert(dst.index());
        }

        self.tmp_flac
            .extend_terminals(input.requirements[required_node_ids].iter().copied());
        self.tmp_flac.run(&input.graph, &mut self.tmp_steiner_tree);

        self.tmp_steiner_tree.total_weight
    }
}
