use std::borrow::Cow;

use operation::OperationContext;
use petgraph::dot::{Config, Dot};
use walker::Walk;

use crate::dot_graph::Attrs;

use super::{Edge, Node, Query, QueryField, SolutionGraph};

#[allow(unused)]
impl<Step> Query<SolutionGraph, Step> {
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

impl Node {
    fn label<Step>(&self, query: &Query<SolutionGraph, Step>, ctx: OperationContext<'_>) -> Attrs<'static> {
        Attrs::label(match self {
            Node::Root => "root".into(),
            Node::QueryPartition {
                resolver_definition_id, ..
            } => resolver_definition_id.walk(ctx.schema).name().into(),
            Node::Field { id, .. } => {
                let field = field_label(query, ctx, &query[*id]);
                format!(
                    "{}{}",
                    if query[*id].query_position.is_none() { "*" } else { "" },
                    field
                )
            }
            Node::Typename => "__typename".into(),
        })
    }

    /// Meant to be as readable as possible for large graphs with colors.
    fn pretty_label<Step>(&self, query: &Query<SolutionGraph, Step>, ctx: OperationContext<'_>) -> String {
        self.label(query, ctx)
            .with_if(
                matches!(self, Node::QueryPartition { .. }),
                "color=royalblue,shape=parallelogram",
            )
            .to_string()
    }
}

impl Edge {
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
    if let Some(key) = field.response_key {
        key.walk(ctx).into()
    } else {
        field.definition_id.walk(ctx).name().into()
    }
}

pub(crate) fn field_label<'a, G: petgraph::visit::GraphBase, S>(
    query: &'a Query<G, S>,
    ctx: OperationContext<'a>,
    field: &QueryField,
) -> Cow<'a, str> {
    let definition = field.definition_id.walk(ctx);
    let alias = if let Some(alias) = field.response_key.walk(ctx).filter(|key| *key != definition.name()) {
        format!("{}: ", alias)
    } else {
        String::new()
    };
    let common = format!("{}.{}", definition.parent_entity().name(), definition.name());
    let subgraph_key = if let Some((_, subgraph_key)) = field
        .response_key
        .zip(field.subgraph_key)
        .filter(|(key, subgraph_key)| key != subgraph_key)
    {
        format!(" ({})", subgraph_key.walk(ctx))
    } else {
        String::new()
    };
    let mut tyc = String::new();
    if !query[field.type_conditions].is_empty() {
        tyc.push_str("on");
        for ty in query[field.type_conditions].walk(ctx) {
            tyc.push_str(&format!(" {}", ty.name()));
        }
        tyc.push(' ');
    }
    Cow::Owned(format!("{tyc}{alias}{common}{subgraph_key}"))
}
