mod builder;
mod edge;
mod node;

use operation::{Operation, OperationContext};
use schema::{FieldSetRecord, Schema};
use std::{borrow::Cow, num::NonZero};
use tracing::{Level, instrument};

use petgraph::{
    dot::{Config, Dot},
    stable_graph::StableGraph,
    visit::GraphBase,
};

use crate::Query;
pub(crate) use edge::*;
pub(crate) use node::*;

pub(crate) type SolutionSpaceGraph<'schema> = StableGraph<SpaceNode, SpaceEdge>;
pub(crate) type SpaceNodeId = <SolutionSpaceGraph<'static> as GraphBase>::NodeId;
pub(crate) type SpaceEdgeId = <SolutionSpaceGraph<'static> as GraphBase>::EdgeId;
pub(crate) type QuerySolutionSpace<'schema> = Query<SolutionSpaceGraph<'schema>, SolutionSpace<'schema>>;

#[derive(Default, id_derives::IndexedFields)]
pub(crate) struct SolutionSpace<'schema> {
    #[indexed_by(SpaceFieldSetId)]
    pub field_sets: Vec<Cow<'schema, FieldSetRecord>>,
    #[indexed_by(DeriveId)]
    pub derive: Vec<Derive>,
}

#[derive(id_derives::Id, Clone, Copy)]
pub struct SpaceFieldSetId(NonZero<u32>);

#[derive(id_derives::Id, Clone, Copy)]
pub struct DeriveId(NonZero<u32>);

impl<'schema> SolutionSpace<'schema> {
    pub fn push_field_set_ref(&mut self, field_set: &'schema FieldSetRecord) -> SpaceFieldSetId {
        self.field_sets.push(Cow::Borrowed(field_set));
        SpaceFieldSetId::from(self.field_sets.len() - 1)
    }
    pub fn push_field_set(&mut self, field_set: Cow<'schema, FieldSetRecord>) -> SpaceFieldSetId {
        self.field_sets.push(field_set);
        SpaceFieldSetId::from(self.field_sets.len() - 1)
    }
    pub fn push_derive(&mut self, derive: Derive) -> DeriveId {
        self.derive.push(derive);
        DeriveId::from(self.derive.len() - 1)
    }
}

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
