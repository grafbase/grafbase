/// Query solving module
///
/// Reduces the query solution space down to a single graph representing the query plan.
///
/// This module has three main steps:
///
/// 1. Generate the graph on which we solve the steiner tree problem. The query solution space
///    can be big and many edges & nodes are not relevant. We only need to keep those where a
///    choice needs to be made, i.e. where the query can be resolved in multiple ways. If a field
///    is only available through one resolver, we don't need to keep track of it.
/// 2. Solve the (steiner) graph. Here we have a two phases algorithm:
///    a. Tree Growth Phase (FLAC): We grow the steiner tree with a partial solution. For this rely
///    on the GreedyFLAC algorithm, which itself just applies the FLAC algorithm repeatedly.
///    b. Weight Update Phase (Fixed-Point): We update the edge weights based on their requirements
///    and the current state of the tree. This is done until we reach a fixed point where the
///    weights don't change anymore.
/// 3. Use the solution to filter out unnecessary edges and nodes of the query solution space into
///    the crude solved query.
///
mod input;
mod solution;
mod solver;
mod steiner_tree;
mod updater;

pub(crate) use solution::*;
pub(crate) use solver::*;

use crate::{Query, SolutionGraph};

pub(crate) type QuerySteinerSolution = Query<SolutionGraph, crate::steps::SteinerSolution>;
