mod input;
mod solution;
mod solver;
mod steiner_tree;
mod updater;

pub(crate) use solution::*;
pub(crate) use solver::*;

use crate::{Query, SolutionGraph};

pub(crate) type CrudeSolvedQuery = Query<SolutionGraph, crate::query::steps::SteinerTreeSolution>;
