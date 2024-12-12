mod builder;
mod edge;
mod node;

pub(crate) use edge::*;
pub(crate) use node::*;

use operation::{Operation, OperationContext};
use schema::Schema;
use tracing::{instrument, Level};

use petgraph::{
    dot::{Config, Dot},
    stable_graph::StableGraph,
};

use crate::Query;

pub(crate) type QuerySolutionSpace<'schema> =
    Query<StableGraph<SpaceNode<'schema>, SpaceEdge>, crate::query::steps::SolutionSpace>;

impl<'schema> QuerySolutionSpace<'schema> {
    #[instrument(skip_all, level = Level::DEBUG)]
    pub fn generate_solution_space<'op>(schema: &'schema Schema, operation: &'op Operation) -> crate::Result<Self>
    where
        'schema: 'op,
    {
        QuerySolutionSpace::builder(schema, operation).build().inspect(|query| {
            tracing::debug!(
                "OperationGraph created:\n{}",
                query.to_pretty_dot_graph(OperationContext { schema, operation })
            );
        })
    }

    /// Use https://dreampuf.github.io/GraphvizOnline
    /// or `echo '..." | dot -Tsvg` from graphviz
    pub(crate) fn to_pretty_dot_graph(&self, ctx: OperationContext<'_>) -> String {
        format!(
            "{:?}",
            Dot::with_attr_getters(
                &self.graph,
                &[Config::EdgeNoLabel, Config::NodeNoLabel],
                &|_, edge| edge.weight().pretty_label().to_string(),
                &|_, node| node.1.pretty_label(self, ctx).to_string()
            )
        )
    }

    /// Use https://dreampuf.github.io/GraphvizOnline
    /// or `echo '..." | dot -Tsvg` from graphviz
    #[cfg(test)]
    pub(crate) fn to_dot_graph(&self, ctx: OperationContext<'_>) -> String {
        format!(
            "{:?}",
            Dot::with_attr_getters(
                &self.graph,
                &[Config::EdgeNoLabel, Config::NodeNoLabel],
                &|_, edge| {
                    let label: &'static str = edge.weight().into();
                    crate::dot_graph::Attrs::label(label).to_string()
                },
                &|_, node| node.1.label(self, ctx).to_string(),
            )
        )
    }
}

impl std::fmt::Debug for QuerySolutionSpace<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Query").finish_non_exhaustive()
    }
}
