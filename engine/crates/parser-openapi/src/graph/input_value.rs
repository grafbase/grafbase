use engine::Name;
use engine_value::ConstValue;
use inflector::Inflector;
use petgraph::{graph::NodeIndex, visit::EdgeRef};
use serde_json::Value;

use super::{DebugNode, Edge, Enum, InputObject, Node, OpenApiGraph, ScalarKind, WrappingType};

#[derive(Clone, Debug)]
pub struct InputValue {
    index: NodeIndex,
    wrapping: WrappingType,
}

#[derive(Clone, Copy, Debug)]
pub enum InputValueKind {
    Scalar,
    InputObject,
    Enum,
    Union,
}

impl InputValue {
    pub(super) fn from_index(index: NodeIndex, wrapping: WrappingType, graph: &OpenApiGraph) -> Option<Self> {
        match &graph.graph[index] {
            Node::Object | Node::Scalar(_) | Node::Enum { .. } | Node::Union | Node::PlaceholderType => {
                Some(InputValue { index, wrapping })
            }
            Node::Schema(_) => {
                let type_edge = graph
                    .graph
                    .edges(index)
                    .find(|edge| matches!(edge.weight(), Edge::HasType { .. }))?;

                let Edge::HasType {
                    wrapping: edge_wrapping,
                } = type_edge.weight()
                else {
                    unreachable!()
                };

                // The HasType edge can introduce more wrapping so we need to make sure to account
                // for that.
                let wrapping = wrapping.wrap_with(edge_wrapping.clone());

                let inner_index = type_edge.target();

                InputValue::from_index(inner_index, wrapping, graph)
            }
            Node::Operation(_)
            | Node::Default(_)
            | Node::UnionWrappedScalar(_)
            | Node::PossibleValue(_)
            | Node::AllOf => None,
        }
    }

    pub fn kind(&self, graph: &OpenApiGraph) -> Option<InputValueKind> {
        match &graph.graph[self.index] {
            Node::Scalar(_) => Some(InputValueKind::Scalar),
            Node::Object => Some(InputValueKind::InputObject),
            Node::Enum { .. } => Some(InputValueKind::Enum),
            Node::Union => Some(InputValueKind::Union),
            Node::Schema(_)
            | Node::Operation(_)
            | Node::Default(_)
            | Node::PossibleValue(_)
            | Node::UnionWrappedScalar(_)
            | Node::AllOf
            | Node::PlaceholderType => None,
        }
    }

    pub fn type_name(&self, graph: &OpenApiGraph) -> Option<String> {
        match &graph.graph[self.index] {
            Node::Scalar(s) => Some(s.type_name()),
            Node::Enum { .. } => Enum::from_index(self.index, graph)?.name(graph),
            Node::Object | Node::Union => InputObject::from_index(self.index, graph)?.name(graph),
            Node::PlaceholderType => {
                // Any placeholders that make it this far should just be mapped to JSON.
                Some(ScalarKind::Json.type_name())
            }
            Node::Schema(_)
            | Node::Operation(_)
            | Node::Default(_)
            | Node::PossibleValue(_)
            | Node::UnionWrappedScalar(_)
            | Node::AllOf => {
                // These shouldn't really happen
                None
            }
        }
    }

    pub fn as_input_object(&self, graph: &OpenApiGraph) -> Option<InputObject> {
        InputObject::from_index(self.index, graph)
    }

    pub fn wrapping_type(&self) -> &WrappingType {
        &self.wrapping
    }

    pub fn default_value(&self, graph: &OpenApiGraph) -> Option<ConstValue> {
        graph
            .graph
            .neighbors(self.index)
            .find_map(|index| match &graph.graph[index] {
                Node::Default(value) => Some(value.clone()),
                _ => None,
            })
            .and_then(|value| self.transform_default(value, graph))
    }

    /// We get a serde_json::Value default value from OpenAPI, which is nice because it's almost
    /// what we need.  But, we're not representing things exactly the same way in GraphQL as they
    /// are in the underlying OpenAPI spec, so we need to do a bunch of transformations on the
    /// value before using it.
    fn transform_default(&self, value: Value, graph: &OpenApiGraph) -> Option<ConstValue> {
        #[allow(clippy::match_same_arms)]
        match (self.kind(graph)?, value) {
            (InputValueKind::Scalar, value) => {
                // just going to pass scalars straight through, we don't
                // tend to transform them
                Some(ConstValue::from_json(value).ok()?)
            }
            (_, Value::Null) => {
                // Nulls also need no transformation
                Some(ConstValue::Null)
            }
            (_, Value::Array(values)) => Some(ConstValue::List(
                values
                    .into_iter()
                    .map(|inner_value| self.transform_default(inner_value, graph))
                    .collect::<Option<_>>()?,
            )),
            (InputValueKind::InputObject, Value::Object(mut object)) => Some(ConstValue::Object(
                self.as_input_object(graph)?
                    .fields(graph)
                    .into_iter()
                    .filter_map(|field| Some((object.remove(field.name.openapi_name())?, field)))
                    .map(|(inner_value, field)| {
                        Some((
                            Name::new(field.name.to_string()),
                            field.value_type.transform_default(inner_value, graph)?,
                        ))
                    })
                    .collect::<Option<_>>()?,
            )),
            (InputValueKind::InputObject, _) => None,
            (InputValueKind::Enum, Value::String(value)) => {
                Some(ConstValue::Enum(Name::new(value.to_screaming_snake_case())))
            }
            (InputValueKind::Enum, _) => {
                // we only support string enums
                None
            }
            (InputValueKind::Union, _) => {
                // Input unions are kind of tricky since they have
                // no direct equivalent in GQL.  It might be a oneOf but
                // for now I'm going to skip
                None
            }
        }
    }
}

impl DebugNode for InputValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, graph: &OpenApiGraph) -> std::fmt::Result {
        f.debug_struct("InputValue")
            .field("kind", &self.kind(graph))
            .field("wrapping_type", self.wrapping_type())
            .finish_non_exhaustive()
    }
}
