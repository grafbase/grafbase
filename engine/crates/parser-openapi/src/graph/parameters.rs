use std::borrow::Cow;

use petgraph::graph::EdgeIndex;
use registry_v2::resolvers::http::{QueryParameterEncodingStyle, RequestBodyContentType};

use super::{input_value::InputValue, DebugNode, Edge, FieldName};

#[derive(Clone, Copy)]
pub struct PathParameter(pub(super) EdgeIndex);

#[derive(Clone, Copy)]
pub struct QueryParameter(pub(super) EdgeIndex);

#[derive(Clone, Copy)]
pub struct RequestBody(pub(super) EdgeIndex);

impl PathParameter {
    pub fn openapi_name(self, graph: &super::OpenApiGraph) -> &str {
        match graph.graph.edge_weight(self.0) {
            Some(Edge::HasPathParameter { name, .. }) => name,
            _ => unreachable!(),
        }
    }

    pub fn graphql_name(self, graph: &super::OpenApiGraph) -> FieldName<'_> {
        match graph.graph.edge_weight(self.0) {
            Some(Edge::HasPathParameter { name, .. }) => FieldName(Cow::Borrowed(name)),
            _ => unreachable!(),
        }
    }

    pub fn input_value(self, graph: &super::OpenApiGraph) -> Option<InputValue> {
        let (_, dest_index) = graph.graph.edge_endpoints(self.0)?;
        match graph.graph.edge_weight(self.0)? {
            Edge::HasPathParameter { wrapping, .. } => InputValue::from_index(dest_index, wrapping.clone(), graph),
            _ => None,
        }
    }
}

impl DebugNode for PathParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, graph: &super::OpenApiGraph) -> std::fmt::Result {
        f.debug_struct("PathParameter")
            .field("openapi_name", &self.openapi_name(graph))
            .field("graphql_name", &self.graphql_name(graph).to_string())
            .field(
                "input_value",
                &self.input_value(graph).as_ref().map(|value| value.debug(graph)),
            )
            .finish()
    }
}

impl QueryParameter {
    pub fn openapi_name(self, graph: &super::OpenApiGraph) -> &str {
        match graph.graph.edge_weight(self.0) {
            Some(Edge::HasQueryParameter { name, .. }) => name,
            _ => unreachable!(),
        }
    }

    pub fn graphql_name(self, graph: &super::OpenApiGraph) -> FieldName<'_> {
        match graph.graph.edge_weight(self.0) {
            Some(Edge::HasQueryParameter { name, .. }) => FieldName(Cow::Borrowed(name)),
            _ => unreachable!(),
        }
    }

    pub fn input_value(self, graph: &super::OpenApiGraph) -> Option<InputValue> {
        let (_, dest_index) = graph.graph.edge_endpoints(self.0)?;
        match graph.graph.edge_weight(self.0)? {
            Edge::HasQueryParameter { wrapping, .. } => InputValue::from_index(dest_index, wrapping.clone(), graph),
            _ => None,
        }
    }

    pub fn encoding_style(self, graph: &super::OpenApiGraph) -> Option<QueryParameterEncodingStyle> {
        match graph.graph.edge_weight(self.0)? {
            Edge::HasQueryParameter { encoding_style, .. } => Some(*encoding_style),
            _ => None,
        }
    }
}

impl RequestBody {
    pub fn argument_name(self) -> &'static str {
        "input"
    }

    pub fn input_value(self, graph: &super::OpenApiGraph) -> Option<InputValue> {
        let (_, dest_index) = graph.graph.edge_endpoints(self.0)?;
        match graph.graph.edge_weight(self.0)? {
            Edge::HasRequestType { wrapping, .. } => InputValue::from_index(dest_index, wrapping.clone(), graph),
            _ => None,
        }
    }

    pub fn content_type(self, graph: &super::OpenApiGraph) -> &RequestBodyContentType {
        match graph.graph.edge_weight(self.0).unwrap() {
            Edge::HasRequestType { content_type, .. } => content_type.as_ref(),
            _ => {
                unreachable!()
            }
        }
    }
}
