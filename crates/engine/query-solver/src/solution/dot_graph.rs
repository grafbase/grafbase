use petgraph::dot::{Config, Dot};
use walker::Walk;

use crate::{dot_graph::Attrs, FieldFlags, Operation};

use super::{Solution, SolutionEdge, SolutionNode};

#[allow(unused)]
impl<'ctx, Op: Operation> Solution<'ctx, Op> {
    /// Use https://dreampuf.github.io/GraphvizOnline
    /// or `echo '..." | dot -Tsvg` from graphviz
    pub(crate) fn to_pretty_dot_graph(&self) -> String {
        format!(
            "{:?}",
            Dot::with_attr_getters(
                &self.graph,
                &[Config::EdgeNoLabel, Config::NodeNoLabel],
                &|_, edge| edge.weight().pretty_label(),
                &|_, node| node.1.pretty_label(self).to_string()
            )
        )
    }

    /// Use https://dreampuf.github.io/GraphvizOnline
    /// or `echo '..." | dot -Tsvg` from graphviz
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

impl<FieldId> SolutionNode<FieldId> {
    fn label<Op: Operation<FieldId = FieldId>>(&self, graph: &Solution<'_, Op>) -> Attrs<'static>
    where
        FieldId: Copy,
    {
        Attrs::label(match self {
            SolutionNode::Root => "root".into(),
            SolutionNode::QueryPartition {
                resolver_definition_id, ..
            } => resolver_definition_id.walk(graph.schema).name().into(),
            SolutionNode::Field { id, flags, .. } => {
                let field = graph.operation.field_label(*id);
                format!("{}{}", if flags.contains(FieldFlags::EXTRA) { "*" } else { "" }, field)
            }
        })
    }

    /// Meant to be as readable as possible for large graphs with colors.
    fn pretty_label<Op: Operation<FieldId = FieldId>>(&self, graph: &Solution<'_, Op>) -> String
    where
        FieldId: Copy,
    {
        self.label(graph)
            .with_if(
                matches!(self, SolutionNode::QueryPartition { .. }),
                "color=royalblue,shape=parallelogram",
            )
            .to_string()
    }
}

impl SolutionEdge {
    /// Meant to be as readable as possible for large graphs with colors.
    fn pretty_label(&self) -> String {
        match self {
            Self::QueryPartition => Attrs::default().with("color=royalblue,fontcolor=royalblue"),
            Self::Field => Attrs::default(),
            Self::RequiredBySubgraph => Attrs::default().with("color=orangered,arrowhead=inv"),
            Self::RequiredBySupergraph => Attrs::default().with("color=orangered,arrowhead=inv,style=dashed"),
            Self::MutationExecutedAfter => Attrs::default().with("color=red,arrowhead=inv,style=dashed"),
        }
        .to_string()
    }
}
