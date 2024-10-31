mod builder;
mod edge;
mod node;
mod prune;
mod solve;

pub(crate) use edge::*;
pub(crate) use node::*;

use schema::{FieldDefinitionId, RequiredField, Schema};
use tracing::instrument;

use std::borrow::Cow;

use petgraph::{
    dot::{Config, Dot},
    stable_graph::{NodeIndex, StableGraph},
};

use crate::dot_graph::Attrs;

pub type Cost = u16;

pub trait Operation {
    type FieldId: From<usize> + Into<usize> + Copy + std::fmt::Debug + Ord;

    fn field_ids(&self) -> impl ExactSizeIterator<Item = Self::FieldId> + 'static;
    fn field_defintion(&self, field_id: Self::FieldId) -> Option<FieldDefinitionId>;
    fn field_satisfies(&self, field_id: Self::FieldId, requirement: RequiredField<'_>) -> bool;
    fn create_extra_field(
        &mut self,
        petitioner_field_id: Self::FieldId,
        requirement: RequiredField<'_>,
    ) -> Self::FieldId;

    fn root_selection_set(&self) -> impl ExactSizeIterator<Item = Self::FieldId> + '_;
    fn subselection(&self, field_id: Self::FieldId) -> impl ExactSizeIterator<Item = Self::FieldId> + '_;

    fn field_label(&self, field_id: Self::FieldId) -> Cow<'_, str>;
}

pub struct OperationGraph<'ctx, Op: Operation> {
    pub(crate) schema: &'ctx Schema,
    pub(crate) operation: Op,
    root_ix: NodeIndex,
    pub(crate) graph: StableGraph<Node<Op::FieldId>, Edge>,
}

impl<'ctx, Op: Operation> OperationGraph<'ctx, Op> {
    #[instrument(skip_all)]
    pub fn new(schema: &'ctx Schema, operation: Op) -> crate::Result<OperationGraph<'ctx, Op>> {
        Self::builder(schema, operation).build().inspect(|op| {
            tracing::debug!("OperationGraph created:\n{}", op.to_pretty_dot_graph());
        })
    }

    pub fn solver(&mut self) -> crate::Result<solve::Solver<'_, 'ctx, Op>> {
        solve::Solver::initialize(self)
    }

    /// Use https://dreampuf.github.io/GraphvizOnline
    /// or `echo '..." | dot -Tsvg` from graphviz
    pub fn to_pretty_dot_graph(&self) -> String {
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
    pub fn to_dot_graph(&self) -> String {
        format!(
            "{:?}",
            Dot::with_attr_getters(
                &self.graph,
                &[Config::EdgeNoLabel, Config::NodeNoLabel],
                &|_, edge| {
                    let label: &'static str = edge.weight().into();
                    Attrs::label(label).to_string()
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
