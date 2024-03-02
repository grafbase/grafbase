use std::collections::BTreeMap;

use engine_parser::{
    types::{OperationDefinition, OperationType, Selection},
    Positioned,
};
use engine_v2_common::HttpGraphqlResponse;
use graph_entities::QueryResponse;
use http::{
    header::{HeaderMap, HeaderName},
    HeaderValue,
};
use serde::{ser::SerializeMap, Deserialize, Serialize};
pub use streaming::*;

use crate::{CacheControl, QueryEnv, Result, ServerError, Value};

mod streaming;

/// GraphQL operation used in the request.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ResponseOperation {
    pub name: Option<String>,
    pub r#type: common_types::OperationType,
}

/// Query response
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Response {
    /// Data of query result
    #[serde(default)]
    pub data: QueryResponse,

    /// Extensions result
    #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
    pub extensions: BTreeMap<String, Value>,

    /// Cache control value
    pub cache_control: CacheControl,

    /// Errors
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub errors: Vec<ServerError>,

    /// HTTP headers
    #[serde(skip)]
    pub http_headers: HeaderMap,

    /// GraphQL operation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub graphql_operation: Option<ResponseOperation>,
}

impl QueryEnv {
    pub fn response_operation(&self) -> ResponseOperation {
        let operation_name =
            self.operation_name
                .as_deref()
                .or_else(|| match self.operation.selection_set.node.items.as_slice() {
                    [Positioned {
                        node: Selection::Field(field),
                        ..
                    }] => Some(field.node.name.node.as_str()),
                    _ => None,
                });
        ResponseOperation {
            name: operation_name.map(str::to_owned),
            r#type: response_operation_for_definition(&self.operation),
        }
    }
}

fn response_operation_for_definition(operation: &OperationDefinition) -> common_types::OperationType {
    match operation.ty {
        OperationType::Query => common_types::OperationType::Query {
            is_introspection: is_operation_introspection(operation),
        },
        OperationType::Mutation => common_types::OperationType::Mutation,
        OperationType::Subscription => common_types::OperationType::Subscription,
    }
}

fn is_operation_introspection(operation: &OperationDefinition) -> bool {
    operation.ty == OperationType::Query
        && operation
            .selection_set
            .node
            .items
            .iter()
            // If field name starts with `__` it is part of introspection system, see http://spec.graphql.org/October2021/#sec-Names.Reserved-Names
            .all(|item| {
                matches!(
                    &item.node,
                    Selection::Field(field) if field.node.name.node.starts_with("__") || field.node.name.node == "_service"
                )
            })
}

impl Response {
    pub fn to_graphql_response(&self) -> GraphQlResponse {
        GraphQlResponse(self)
    }

    /// Create a new successful response with the data.
    #[must_use]
    pub fn new(mut data: QueryResponse, response_operation: ResponseOperation) -> Self {
        data.shrink_to_fit();
        Self {
            data,
            graphql_operation: Some(response_operation),
            ..Default::default()
        }
    }

    /// Create a new successful response with the data.
    #[cfg(test)]
    #[must_use]
    pub fn new_test(mut data: QueryResponse) -> Self {
        data.shrink_to_fit();
        Self {
            data,
            graphql_operation: None,
            ..Default::default()
        }
    }

    #[must_use]
    pub fn bad_request(errors: Vec<ServerError>) -> Self {
        Self {
            errors,
            graphql_operation: None,
            ..Default::default()
        }
    }

    /// Create a response from some errors.
    #[must_use]
    pub fn from_errors_with_type(errors: Vec<ServerError>, operation_type: OperationType) -> Self {
        Self {
            errors,
            graphql_operation: Some(ResponseOperation {
                name: None,
                r#type: match operation_type {
                    OperationType::Query => common_types::OperationType::Query {
                        is_introspection: false,
                    },
                    OperationType::Mutation => common_types::OperationType::Mutation,
                    OperationType::Subscription => common_types::OperationType::Subscription,
                },
            }),
            ..Default::default()
        }
    }

    /// Create a response from some errors.
    #[must_use]
    pub fn from_errors(errors: Vec<ServerError>, response_operation: ResponseOperation) -> Self {
        Self {
            errors,
            graphql_operation: Some(response_operation),
            ..Default::default()
        }
    }

    /// Set the extension result of the response.
    #[must_use]
    pub fn extension(mut self, name: impl Into<String>, value: Value) -> Self {
        self.extensions.insert(name.into(), value);
        self
    }

    /// Set the http headers of the response.
    #[must_use]
    pub fn http_headers(self, http_headers: HeaderMap) -> Self {
        Self { http_headers, ..self }
    }

    /// Set the cache control of the response.
    #[must_use]
    pub fn cache_control(self, cache_control: CacheControl) -> Self {
        Self { cache_control, ..self }
    }

    /// Returns `true` if the response is ok.
    #[inline]
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }

    /// Returns `true` if the response is error.
    #[inline]
    pub fn is_err(&self) -> bool {
        !self.is_ok()
    }

    /// Extract the error from the response. Only if the `error` field is empty will this return
    /// `Ok`.
    #[inline]
    pub fn into_result(self) -> Result<Self, Vec<ServerError>> {
        if self.is_err() {
            Err(self.errors)
        } else {
            Ok(self)
        }
    }
}

/// Response for batchable queries
#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum BatchResponse {
    /// Response for single queries
    Single(Response),

    /// Response for batch queries
    Batch(Vec<Response>),
}

impl BatchResponse {
    /// Gets cache control value
    pub fn cache_control(&self) -> CacheControl {
        match self {
            BatchResponse::Single(resp) => resp.cache_control.clone(),
            BatchResponse::Batch(resp) => resp.iter().fold(CacheControl::default(), |mut acc, item| {
                acc.merge(item.cache_control.clone());
                acc
            }),
        }
    }

    /// Returns `true` if all responses are ok.
    pub fn is_ok(&self) -> bool {
        match self {
            BatchResponse::Single(resp) => resp.is_ok(),
            BatchResponse::Batch(resp) => resp.iter().all(Response::is_ok),
        }
    }

    /// Returns HTTP headers map.
    pub fn http_headers(&self) -> HeaderMap {
        match self {
            BatchResponse::Single(resp) => resp.http_headers.clone(),
            BatchResponse::Batch(resp) => resp.iter().fold(HeaderMap::new(), |mut acc, resp| {
                acc.extend(resp.http_headers.clone());
                acc
            }),
        }
    }

    /// Returns HTTP headers iterator.
    pub fn http_headers_iter(&self) -> impl Iterator<Item = (HeaderName, HeaderValue)> {
        let headers = self.http_headers();

        let mut current_name = None;
        headers.into_iter().filter_map(move |(name, value)| {
            if let Some(name) = name {
                current_name = Some(name);
            }
            current_name.clone().map(|current_name| (current_name, value))
        })
    }

    pub fn into_json_value(self) -> serde_json::Result<serde_json::Value> {
        match self {
            Self::Batch(multiple) => {
                serde_json::to_value(multiple.iter().map(Response::to_graphql_response).collect::<Vec<_>>())
            }
            Self::Single(single) => serde_json::to_value(single.to_graphql_response()),
        }
    }
}

impl From<Response> for BatchResponse {
    fn from(response: Response) -> Self {
        Self::Single(response)
    }
}

impl From<Vec<Response>> for BatchResponse {
    fn from(responses: Vec<Response>) -> Self {
        Self::Batch(responses)
    }
}

/// A wrapper around Response that Serialises in GraphQL format
pub struct GraphQlResponse<'a>(&'a Response);

impl serde::Serialize for GraphQlResponse<'_> {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(5))?;
        map.serialize_entry("data", &self.0.data.as_graphql_data())?;
        if !self.0.errors.is_empty() {
            map.serialize_entry("errors", &self.0.errors)?;
        }
        if !self.0.extensions.is_empty() {
            map.serialize_entry("extensions", &self.0.extensions)?;
        }
        map.end()
    }
}

impl Response {
    pub fn into_json_bytes(&self) -> Result<Vec<u8>, Vec<u8>> {
        serde_json::to_vec(&self.to_graphql_response()).map_err(|err| {
            tracing::error!("Failed to serialize response: {}", err);
            serde_json::to_vec(&serde_json::json!({
                "errors": [
                    {"message": "Internal server error"}
                ]
            }))
            .unwrap()
        })
    }
}

impl From<Response> for HttpGraphqlResponse {
    fn from(value: Response) -> Self {
        match value.into_json_bytes() {
            Ok(body) => Self::from_bytes(body.into()),
            Err(body) => Self::from_bytes(body.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_response_single() {
        let json = Value::Boolean(true).into_json().unwrap();
        let mut resp = QueryResponse::default();
        let id = resp.from_serde_value(json);
        resp.set_root_unchecked(id);

        let resp = Response::new_test(resp);

        let resp = BatchResponse::Single(resp);
        assert_eq!(resp.into_json_value().unwrap().to_string(), r#"{"data":true}"#);
    }

    #[test]
    fn test_batch_response_batch() {
        let json1 = Value::Boolean(true).into_json().unwrap();
        let mut resp1 = QueryResponse::default();
        let id = resp1.from_serde_value(json1);
        resp1.set_root_unchecked(id);
        let resp1 = Response::new_test(resp1);

        let json2 = Value::String("1".to_string()).into_json().unwrap();
        let mut resp2 = QueryResponse::default();
        let id = resp2.from_serde_value(json2);
        resp2.set_root_unchecked(id);
        let resp2 = Response::new_test(resp2);

        let resp = BatchResponse::Batch(vec![resp1, resp2]);
        assert_eq!(
            resp.into_json_value().unwrap().to_string(),
            r#"[{"data":true},{"data":"1"}]"#
        );
    }
}
