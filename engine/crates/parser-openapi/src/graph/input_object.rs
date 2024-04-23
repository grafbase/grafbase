use std::borrow::Cow;

use petgraph::{
    graph::NodeIndex,
    visit::{Dfs, EdgeFiltered, EdgeRef, Walker},
};

use super::{Edge, FieldName, InputValue, Node};

#[derive(Clone, Copy, Debug)]
pub struct InputObject {
    index: NodeIndex,
    one_of: bool,
}

#[derive(Debug)]
pub struct InputField<'a> {
    pub value_type: InputValue,
    pub name: FieldName<'a>,
}

impl InputObject {
    pub(super) fn from_index(index: NodeIndex, graph: &super::OpenApiGraph) -> Option<Self> {
        match graph.graph[index] {
            Node::Object => Some(InputObject { index, one_of: false }),
            Node::Union => Some(InputObject { index, one_of: true }),
            Node::Schema(_) => InputObject::from_index(graph.schema_target(index)?, graph),
            Node::Operation(_)
            | Node::AllOf
            | Node::Scalar(_)
            | Node::Enum { .. }
            | Node::Default(_)
            | Node::PossibleValue(_)
            | Node::UnionWrappedScalar(_)
            | Node::PlaceholderType => None,
        }
    }

    pub fn name(self, graph: &super::OpenApiGraph) -> Option<String> {
        // Unlike in GraphQL, OpenAPI objects can appear in both input & output position.
        // To support this we append Input to whatever name we'd otherwise pick for this
        // object.
        Some(format!("{}Input", graph.type_name(self.index)?))
    }

    pub fn one_of(self) -> bool {
        self.one_of
    }

    pub fn fields(self, graph: &super::OpenApiGraph) -> Vec<InputField<'_>> {
        graph
            .graph
            .edges(self.index)
            .filter_map(|edge| match edge.weight() {
                super::Edge::HasField {
                    name,
                    wrapping,
                    required,
                } => Some(InputField {
                    value_type: InputValue::from_index(edge.target(), wrapping.clone().set_required(*required), graph)?,
                    name: FieldName(Cow::Borrowed(name)),
                }),
                super::Edge::HasUnionMember => {
                    let value_type = InputValue::from_index(edge.target(), super::WrappingType::Named, graph)?;
                    let name = value_type.type_name(graph)?;

                    Some(InputField {
                        value_type,
                        name: FieldName(Cow::Owned(name)),
                    })
                }
                _ => None,
            })
            .collect()
    }
}

impl super::OpenApiGraph {
    pub fn input_objects(&self) -> Vec<InputObject> {
        // Don't follow the HasResponseType edge, as nothing in there is an input object.
        let filtered_graph = EdgeFiltered::from_fn(&self.graph, |edge| {
            !matches!(edge.weight(), Edge::HasResponseType { .. })
        });
        let mut dfs = Dfs::empty(&filtered_graph);
        dfs.stack = self.operations().into_iter().map(|op| op.node_index()).collect();

        dfs.iter(&filtered_graph)
            .filter_map(|idx| InputObject::from_index(idx, self))
            .collect()
    }
}
