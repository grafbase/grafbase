use std::borrow::Cow;

use dynaql::registry::resolvers::http::{QueryParameterEncodingStyle, RequestBodyContentType};
use petgraph::graph::EdgeIndex;

use super::{input_value::InputValue, Edge, FieldName};

#[derive(Clone, Copy)]
pub struct PathParameter(pub(super) EdgeIndex);

#[derive(Clone, Copy)]
pub struct QueryParameter(pub(super) EdgeIndex);

#[derive(Clone, Copy)]
pub struct RequestBody(pub(super) EdgeIndex);

impl PathParameter {
    pub fn openapi_name(self, graph: &super::OpenApiGraph) -> Option<&str> {
        match graph.graph.edge_weight(self.0)? {
            Edge::HasPathParameter { name, .. } => Some(name),
            _ => None,
        }
    }

    pub fn graphql_name(self, graph: &super::OpenApiGraph) -> Option<FieldName<'_>> {
        match graph.graph.edge_weight(self.0)? {
            Edge::HasPathParameter { name, .. } => Some(FieldName(Cow::Borrowed(name))),
            _ => None,
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

impl QueryParameter {
    pub fn openapi_name(self, graph: &super::OpenApiGraph) -> Option<&str> {
        match graph.graph.edge_weight(self.0)? {
            Edge::HasQueryParameter { name, .. } => Some(name),
            _ => None,
        }
    }

    pub fn graphql_name(self, graph: &super::OpenApiGraph) -> Option<FieldName<'_>> {
        match graph.graph.edge_weight(self.0)? {
            Edge::HasQueryParameter { name, .. } => Some(FieldName(Cow::Borrowed(name))),
            _ => None,
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
