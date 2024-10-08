mod builder;
mod edge;
mod node;

pub use edge::*;
pub use node::*;

use schema::{FieldDefinitionId, RequiredField, Schema};
use walker::Walk;

use std::borrow::Cow;

use petgraph::{
    dot::{Config, Dot},
    graph::NodeIndex,
};

pub trait Operation: std::fmt::Debug {
    type FieldId: From<usize> + Into<usize> + Copy + std::fmt::Debug + Ord;

    fn field_ids(&self) -> impl ExactSizeIterator<Item = Self::FieldId> + '_;
    fn field_defintion(&self, field_id: Self::FieldId) -> Option<FieldDefinitionId>;
    fn field_satisfies(&self, field_id: Self::FieldId, requirement: RequiredField<'_>) -> bool;
    fn create_extra_field(&mut self, requirement: RequiredField<'_>) -> Self::FieldId;

    fn root_selection_set(&self) -> impl ExactSizeIterator<Item = Self::FieldId> + '_;
    fn subselection(&self, field_id: Self::FieldId) -> impl ExactSizeIterator<Item = Self::FieldId> + '_;

    fn field_label(&self, field_id: Self::FieldId) -> Cow<'_, str>;
}

pub struct OperationGraph<'ctx, Op: Operation> {
    schema: &'ctx Schema,
    operation: &'ctx mut Op,
    inner: petgraph::stable_graph::StableGraph<Node<Op::FieldId>, Edge>,
    root: NodeIndex,
    field_nodes: Vec<NodeIndex>,
    leaf_nodes: Vec<NodeIndex>,
}

impl<'ctx, Op: Operation> std::ops::Index<Op::FieldId> for OperationGraph<'ctx, Op> {
    type Output = NodeIndex;
    fn index(&self, field_id: Op::FieldId) -> &Self::Output {
        let ix: usize = field_id.into();
        &self.field_nodes[ix]
    }
}

impl<'ctx, Op: Operation> OperationGraph<'ctx, Op> {
    pub fn new(schema: &'ctx Schema, operation: &'ctx mut Op) -> OperationGraph<'ctx, Op> {
        Self::builder(schema, operation).build()
    }

    /// Use https://dreampuf.github.io/GraphvizOnline
    /// or `echo '..." | dot -Tsvg` from graphviz
    pub fn to_dot_graph(&self) -> String {
        let node_str = |_, node_ref: (NodeIndex, &Node<Op::FieldId>)| match node_ref.1 {
            Node::Root => r#"label = "root""#.to_string(),
            Node::Field(id) => format!("label = \"{}\"", self.operation.field_label(*id)),
            Node::FieldResolver(field_resolver) => format!(
                "label = \"{}@{}\",shape=box,style=dashed,color=blue",
                field_resolver.field_definition_id.walk(self.schema).name(),
                field_resolver.resolver_definition_id.walk(self.schema).name()
            ),
            Node::Resolver(resolver) => {
                format!(
                    "label = \"{}\",shape=box,color=blue",
                    resolver.definition_id.walk(self.schema).name()
                )
            }
        };
        format!(
            "{:?}",
            Dot::with_attr_getters(
                &self.inner,
                &[Config::NodeNoLabel],
                &|_, edge| {
                    match edge.weight() {
                        Edge::Resolver(_) | Edge::CanResolveField(_) | Edge::Resolves => "color=blue".to_string(),
                        Edge::Field | Edge::TypenameField => String::new(),
                        Edge::Requires => "color=green".to_string(),
                    }
                },
                &node_str
            )
        )
    }
}
