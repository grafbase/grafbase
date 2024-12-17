use std::borrow::Cow;

use operation::OperationContext;
use petgraph::dot::{Config, Dot};
use walker::Walk;

use crate::{dot_graph::Attrs, FieldFlags};

use super::{QueryField, SolutionEdge, SolutionNode, SolvedQuery};

#[allow(unused)]
impl SolvedQuery {
    /// Use https://dreampuf.github.io/GraphvizOnline
    /// or `echo '..." | dot -Tsvg` from graphviz
    pub(crate) fn to_pretty_dot_graph(&self, ctx: OperationContext<'_>) -> String {
        format!(
            "{:?}",
            Dot::with_attr_getters(
                &self.graph,
                &[Config::EdgeNoLabel, Config::NodeNoLabel],
                &|_, edge| edge.weight().pretty_label(),
                &|_, node| node.1.pretty_label(self, ctx).to_string()
            )
        )
    }

    /// Use https://dreampuf.github.io/GraphvizOnline
    /// or `echo '..." | dot -Tsvg` from graphviz
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

impl SolutionNode {
    fn label(&self, solution: &SolvedQuery, ctx: OperationContext<'_>) -> Attrs<'static> {
        Attrs::label(match self {
            SolutionNode::Root => "root".into(),
            SolutionNode::QueryPartition {
                resolver_definition_id, ..
            } => resolver_definition_id.walk(ctx.schema).name().into(),
            SolutionNode::Field { id, flags, .. } => {
                let field = field_label(ctx, &solution[*id]);
                format!("{}{}", if flags.contains(FieldFlags::EXTRA) { "*" } else { "" }, field)
            }
        })
    }

    /// Meant to be as readable as possible for large graphs with colors.
    fn pretty_label(&self, solution: &SolvedQuery, ctx: OperationContext<'_>) -> String {
        self.label(solution, ctx)
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

pub(crate) fn short_field_label<'a>(ctx: OperationContext<'a>, field: &QueryField) -> Cow<'a, str> {
    if let Some(key) = field.key {
        key.walk(ctx).into()
    } else if let Some(def) = field.definition_id {
        def.walk(ctx).name().into()
    } else {
        "__typename".into()
    }
}

pub(crate) fn field_label<'a>(ctx: OperationContext<'a>, field: &QueryField) -> Cow<'a, str> {
    if let Some(definition) = field.definition_id.walk(ctx) {
        if let Some(alias) = field.key.walk(ctx).filter(|key| *key != definition.name()) {
            Cow::Owned(format!(
                "{}: {}.{}",
                alias,
                definition.parent_entity().name(),
                definition.name()
            ))
        } else {
            Cow::Owned(format!("{}.{}", definition.parent_entity().name(), definition.name()))
        }
    } else {
        field.key.walk(ctx).unwrap_or("__typename").into()
    }
}
