mod builder;
mod edge;
mod node;
mod prune;

pub(crate) use edge::*;
pub(crate) use node::*;

use schema::Schema;
use tracing::{instrument, Level};

use petgraph::{
    dot::{Config, Dot},
    stable_graph::{NodeIndex, StableGraph},
};

use crate::Operation;

pub(crate) struct OperationGraph<'ctx, Op: Operation> {
    pub(crate) schema: &'ctx Schema,
    pub(crate) operation: Op,
    pub(crate) root_ix: NodeIndex,
    pub(crate) graph: StableGraph<Node<'ctx, Op::FieldId>, Edge>,
}

impl<'ctx, Op: Operation> OperationGraph<'ctx, Op> {
    #[instrument(skip_all, level = Level::DEBUG)]
    pub fn new(schema: &'ctx Schema, operation: Op) -> crate::Result<OperationGraph<'ctx, Op>> {
        Self::builder(schema, operation).build().inspect(|op| {
            tracing::debug!("OperationGraph created:\n{}", op.to_pretty_dot_graph());
        })
    }

    /// Use https://dreampuf.github.io/GraphvizOnline
    /// or `echo '..." | dot -Tsvg` from graphviz
    pub(crate) fn to_pretty_dot_graph(&self) -> String {
        format!(
            "{:?}",
            Dot::with_attr_getters(
                &self.graph,
                &[Config::EdgeNoLabel, Config::NodeNoLabel],
                &|_, edge| edge.weight().pretty_label(self),
                &|_, node| node.1.pretty_label(self).to_string()
            )
        )
    }

    /// Use https://dreampuf.github.io/GraphvizOnline
    /// or `echo '..." | dot -Tsvg` from graphviz
    #[cfg(test)]
    pub(crate) fn to_dot_graph(&self) -> String {
        format!(
            "{:?}",
            Dot::with_attr_getters(
                &self.graph,
                &[Config::EdgeNoLabel, Config::NodeNoLabel],
                &|_, edge| {
                    let label: &'static str = edge.weight().into();
                    crate::dot_graph::Attrs::label(label).to_string()
                },
                &|_, node| node.1.label(self).to_string(),
            )
        )
    }
}

impl<'ctx, Op: Operation> std::fmt::Debug for OperationGraph<'ctx, Op> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OperationGraph").finish_non_exhaustive()
    }
}
