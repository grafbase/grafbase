use std::ops::ControlFlow;

use id_newtypes::IdRange;
use operation::OperationContext;
use petgraph::graph::NodeIndex;

use crate::solve::{
    input::{
        FreeRequirementByEdge, FreeRequirementByNode, RequiredSpaceNodeId, RequiredSteinerNodeId, RequirementsByEdge,
        SteinerInput, SteinerWeight, UnavoidableParentSteinerEdgeId,
    },
    steiner_tree::{GreedyFlac, SteinerTree},
};

/// Manages the dynamic weight updates for edges based on their requirements.
///
/// # Purpose
///
/// Some edges have requirements (like needing an ID field) that are only costly if we don't already
/// have them. This updater recalculates edge weights as the Steiner tree grows, making requirements
/// "free" if they're already satisfied by the current tree.
pub(crate) struct RequirementAndWeightUpdater {
    /// Tracks whether requirements are independent of each other.
    /// If true, we can skip the fixed-point iteration since weights don't affect each other.
    independent_requirements: Option<bool>,
    /// Temporary storage for extra terminals to be added to the algorithm.
    /// These are requirements that become mandatory once certain edges are chosen.
    tmp_new_terminals: Vec<NodeIndex>,
    tmp_new_space_terminals: Vec<NodeIndex>,
    /// Temporary Steiner tree used for estimating requirement costs.
    /// We clone the main tree and simulate adding requirements to estimate their weight.
    tmp_steiner_tree: SteinerTree,
    /// Temporary FLAC instance for running cost estimation simulations.
    tmp_flac: GreedyFlac,
}

impl RequirementAndWeightUpdater {
    pub fn new(input: &SteinerInput<'_>) -> crate::Result<Self> {
        Ok(Self {
            independent_requirements: None,
            tmp_new_terminals: Vec::new(),
            tmp_new_space_terminals: Vec::new(),
            tmp_steiner_tree: SteinerTree::new(&input.graph, input.root_node_id, Vec::new()),
            tmp_flac: GreedyFlac::new(&input.graph),
        })
    }

    pub fn initialize(
        &mut self,
        ctx: OperationContext<'_>,
        input: &mut SteinerInput<'_>,
        steiner_tree: &mut SteinerTree,
    ) -> crate::Result<()> {
        // No weights to compute
        if input.requirements.requirements_by_edge.is_empty() {
            self.independent_requirements = Some(true);
            Ok(())
        } else {
            let _ = self.run_fixed_point_weight(ctx, input, steiner_tree)?;
            Ok(())
        }
    }

    pub fn run_fixed_point_weight(
        &mut self,
        ctx: OperationContext<'_>,
        input: &mut SteinerInput<'_>,
        steiner_tree: &mut SteinerTree,
    ) -> crate::Result<ControlFlow<()>> {
        FixedPointWeightAlgorithm {
            ctx,
            state: self,
            steiner_tree,
            input,
            has_updated_weights: false,
        }
        .run()
    }
}

pub(crate) struct FixedPointWeightAlgorithm<'s, 't, 'i, 'schema, 'op> {
    pub ctx: OperationContext<'op>,
    pub state: &'s mut RequirementAndWeightUpdater,
    pub steiner_tree: &'t mut SteinerTree,
    pub input: &'i mut SteinerInput<'schema>,
    has_updated_weights: bool,
}

impl std::ops::Deref for FixedPointWeightAlgorithm<'_, '_, '_, '_, '_> {
    type Target = RequirementAndWeightUpdater;
    fn deref(&self) -> &Self::Target {
        self.state
    }
}

impl std::ops::DerefMut for FixedPointWeightAlgorithm<'_, '_, '_, '_, '_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.state
    }
}

impl<'state> FixedPointWeightAlgorithm<'state, '_, '_, '_, '_> {
    /// Updates the weight of edges based on the requirements of the nodes.
    ///
    /// # Fixed-Point Algorithm
    ///
    /// This implements a fixed-point iteration to handle complex requirement dependencies.
    /// Some requirements are trivially free (like requiring an ID field from the parent resolver),
    /// but others may depend on fields that have their own requirements.
    ///
    /// For complex requirements, we don't know whether the cost of one will impact another.
    /// So we might need N rounds of weight updates to get accurate weights. In mathematics,
    /// a fixed point is a value v for which a function f returns itself: v = f(v).
    /// Here, that means re-applying the weight update algorithm results in the same state.
    /// That's how we know we're finished updating.
    ///
    /// We iterate until weight becomes stable or we exhausted the maximum number of iterations which
    /// likely indicates a requirement cycle.
    pub fn run(mut self) -> crate::Result<ControlFlow<()>> {
        debug_assert!(self.tmp_new_terminals.is_empty() && self.tmp_new_space_terminals.is_empty());
        let mut i = 0;
        loop {
            i += 1;
            self.generate_weight_updates_based_on_requirements();
            let has_updated_weight_this_iteration = std::mem::take(&mut self.has_updated_weights);
            if !has_updated_weight_this_iteration || self.independent_requirements.unwrap_or_default() {
                break;
            }
            if i > 100 {
                return Err(crate::Error::RequirementCycleDetected);
            }
        }
        // If it's the first time we do the fixed point iteration and we didn't do more than 2
        // iterations (one for updating, one for checking nothing changed). It means there is no
        // dependency between requirements weight. So we can skip it in the next iterations.
        self.independent_requirements.get_or_insert(i == 2);

        for id in self.state.tmp_new_space_terminals.drain(..) {
            self.input.space_node_is_terminal.insert(id.index());
        }

        Ok(self
            .steiner_tree
            .extend_terminals(self.state.tmp_new_terminals.drain(..)))
    }

    fn extend_terminals(
        &mut self,
        required_node_ids: IdRange<RequiredSteinerNodeId>,
        required_space_node_ids: IdRange<RequiredSpaceNodeId>,
    ) {
        self.state
            .tmp_new_space_terminals
            .extend(self.input.requirements[required_space_node_ids].iter().copied());
        self.state
            .tmp_new_terminals
            .extend(self.input.requirements[required_node_ids].iter().copied());
    }

    /// Updates edge weights based on their requirements and the current Steiner tree state.
    ///
    /// For free requirements, we just need to include the required nodes as terminals if
    /// necessary.
    /// For the others, we need to re-estimate their weight. It's hard to be really smarter than
    /// re-computing it all the time. We estimate the weight of the extra requirements
    /// by computing weight of adding them to the current Steiner tree plus the base weight of the
    /// edge.
    fn generate_weight_updates_based_on_requirements(&mut self) {
        let mut i = 0;
        while let Some(FreeRequirementByNode {
            node_id,
            required_node_ids,
            required_space_node_ids,
        }) = self.input.requirements.free_requirements_by_node.get(i).copied()
        {
            if self.steiner_tree[node_id] {
                self.extend_terminals(required_node_ids, required_space_node_ids);
                self.input.requirements.free_requirements_by_node.swap_remove(i);
            } else {
                i += 1;
            }
        }

        i = 0;
        while let Some(FreeRequirementByEdge {
            edge_id,
            required_node_ids,
            required_space_node_ids,
        }) = self.input.requirements.free_requirements_by_edge.get(i).copied()
        {
            if self.steiner_tree[edge_id] {
                self.extend_terminals(required_node_ids, required_space_node_ids);
                self.input.requirements.free_requirements_by_node.swap_remove(i);
            } else {
                i += 1;
            }
        }

        i = 0;
        while let Some(RequirementsByEdge {
            unavoidable_parent_edge_ids,
            required_space_node_ids,
            required_node_ids,
            dependent_edge_with_inherent_weight_ids,
        }) = self.input.requirements.requirements_by_edge.get(i).copied()
        {
            if self.input.requirements[dependent_edge_with_inherent_weight_ids]
                .iter()
                .any(|(edge_id, _)| self.steiner_tree[*edge_id])
            {
                // One of the dependent edge is part of the steiner tree, so the requirements
                // aren't optional anymore. Every edge that isn't part of the steiner tree is set back
                // to its inherent weight.
                for &(edge_id, inherent_weight) in &self.input.requirements[dependent_edge_with_inherent_weight_ids] {
                    if !self.steiner_tree[edge_id] {
                        let old = std::mem::replace(&mut self.input.graph[edge_id], inherent_weight);
                        self.has_updated_weights |= old != inherent_weight;
                    }
                }
                self.extend_terminals(required_node_ids, required_space_node_ids);
                self.input.requirements.requirements_by_edge.swap_remove(i);
                continue;
            } else if !required_node_ids.is_empty() {
                // The required nodes in the Steiner Graph may be empty because there's no choice
                // to be made on how to retrieve them. If that's the case, we can skip the weight
                // update.
                let requirements_weight = self.state.estimate_requirements_weight(
                    self.input,
                    self.steiner_tree,
                    unavoidable_parent_edge_ids,
                    required_node_ids,
                );

                tracing::debug!(
                    "Updating requirement cost for edges:\n{}",
                    self.input.requirements[dependent_edge_with_inherent_weight_ids]
                        .iter()
                        .map(|(edge_id, inherent_weight)| {
                            let (src, dst) = self.input.graph.edge_endpoints(*edge_id).unwrap();
                            let src = self.input.map.node_id_to_space_node_id[src.index()];
                            let dst = self.input.map.node_id_to_space_node_id[dst.index()];
                            let new_weight = *inherent_weight + requirements_weight;
                            let old_weight = self.input.graph[*edge_id];
                            format!(
                                "{} -> {} with {} = {} + {} from {}",
                                self.input.space.graph[src].label(&self.input.space, self.ctx),
                                self.input.space.graph[dst].label(&self.input.space, self.ctx),
                                new_weight,
                                inherent_weight,
                                requirements_weight,
                                old_weight
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                );

                // The inherent weight, is the weight of the edge before adding any requirements weight. The current graph may has already
                // some requirement weight added, so we can't use the current value.
                for (edge_id, inherent_weight) in &self.input.requirements[dependent_edge_with_inherent_weight_ids] {
                    let weight = *inherent_weight + requirements_weight;
                    let old = std::mem::replace(&mut self.input.graph[*edge_id], weight);
                    self.has_updated_weights |= old != weight;
                }
            }

            i += 1;
        }
    }
}

impl RequirementAndWeightUpdater {
    /// Estimates the cost of satisfying a set of requirements.
    ///
    /// # Algorithm
    ///
    /// This function computes a Steiner tree for the requirements to estimate their cost.
    /// The edges/nodes of this sub-tree don't matter, but the total cost does.
    ///
    /// 1. Clone the current Steiner tree state
    /// 2. Add the required nodes as new terminals
    /// 3. Mark unavoidable parent edges as already in the tree (they're prerequisites)
    /// 4. Run GreedyFLAC to find the minimum cost to connect all requirements
    /// 5. Return the total weight as the estimated cost
    ///
    /// This cost estimation lets the core loop know that taking certain edges is costly
    /// if they require a bunch of intermediate plans.
    fn estimate_requirements_weight(
        &mut self,
        input: &mut SteinerInput<'_>,
        steiner_tree: &SteinerTree,
        unavoidable_parent_edge_ids: IdRange<UnavoidableParentSteinerEdgeId>,
        required_node_ids: IdRange<RequiredSteinerNodeId>,
    ) -> SteinerWeight {
        self.tmp_flac.reset();
        // TODO: could avoid cloning so much if a single FLAC run is enough.
        if self
            .tmp_steiner_tree
            .clone_from_with_new_terminals(steiner_tree, input.requirements[required_node_ids].iter().copied())
            .is_break()
        {
            // Required nodes are already part of the steiner tree.
            return 0;
        }

        // Unavoidable parent edges are edges we must take to reach the node requiring those new
        // nodes. They might not have been included in the steiner tree yet, so we want to
        // differentiate cases where requirements will be easily providable by a parent resolver
        // from those where we need a separate resolver.
        for &edge_id in &input.requirements[unavoidable_parent_edge_ids] {
            self.tmp_steiner_tree.edges.insert(edge_id.index());
            let (_, dst) = input.graph.edge_endpoints(edge_id).unwrap();
            self.tmp_steiner_tree.nodes.insert(dst.index());
        }

        self.tmp_flac.run(&input.graph, &mut self.tmp_steiner_tree);

        self.tmp_steiner_tree.total_weight
    }
}
