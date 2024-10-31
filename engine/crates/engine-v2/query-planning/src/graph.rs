mod builder;
mod edge;
mod node;
mod prune;
mod solve;

pub use edge::*;
pub use node::*;
#[cfg(test)]
pub(crate) use solve::*;

use schema::{FieldDefinitionId, RequiredField, Schema};
use tracing::{instrument, Level};

use std::borrow::Cow;

use petgraph::{
    dot::{Config, Dot},
    stable_graph::{NodeIndex, StableGraph},
    visit::IntoNodeReferences,
};

pub type Cost = u16;

pub trait Operation {
    type FieldId: From<usize> + Into<usize> + Copy + std::fmt::Debug + Ord;

    fn field_ids(&self) -> impl ExactSizeIterator<Item = Self::FieldId> + 'static;
    fn field_defintion(&self, field_id: Self::FieldId) -> Option<FieldDefinitionId>;
    fn field_satisfies(&self, field_id: Self::FieldId, requirement: RequiredField<'_>) -> bool;
    fn create_potential_extra_field(
        &mut self,
        petitioner_field_id: Self::FieldId,
        requirement: RequiredField<'_>,
    ) -> Self::FieldId;
    fn finalize_selection_set_extra_fields(&mut self, extra: &[Self::FieldId], existing: &[Self::FieldId]);

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
    #[instrument(skip_all, level = Level::DEBUG)]
    pub fn new(schema: &'ctx Schema, operation: Op) -> crate::Result<OperationGraph<'ctx, Op>> {
        Self::builder(schema, operation).build().inspect(|op| {
            tracing::debug!("OperationGraph created:\n{}", op.to_pretty_dot_graph());
        })
    }

    pub fn solve(&mut self) -> crate::Result<()> {
        let solution = solve::Solver::initialize(self)?.solve()?;
        self.finalize_extra_fields(&solution);
        self.graph.retain_nodes(|graph, node| match graph[node] {
            Node::Root => true,
            Node::QueryField(_) | Node::Resolver(_) | Node::ProvidableField(_) => solution.node_bitset[node.index()],
        });
        Ok(())
    }

    fn finalize_extra_fields(&mut self, solution: &solve::Solution) {
        let mut extra_fields = Vec::new();
        let mut existing_fields = Vec::new();
        for node_ix in self.graph.node_references().filter_map(|(node_ix, node)| match node {
            Node::Root => Some(node_ix),
            Node::QueryField(field) if !field.is_scalar() => Some(node_ix),
            _ => None,
        }) {
            extra_fields.clear();
            existing_fields.clear();

            for node_ix in self.graph.neighbors_directed(node_ix, petgraph::Direction::Outgoing) {
                let Node::QueryField(field) = &self.graph[node_ix] else {
                    continue;
                };
                if !solution.node_bitset[node_ix.index()] {
                    continue;
                }
                if field.is_extra() {
                    extra_fields.push(field.id)
                } else {
                    existing_fields.push(field.id);
                }
            }
            self.operation
                .finalize_selection_set_extra_fields(&extra_fields, &existing_fields);
        }
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
