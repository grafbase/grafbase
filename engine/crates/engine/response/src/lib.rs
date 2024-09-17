use std::{collections::BTreeMap, time::Duration};

use engine_parser::types::OperationType;
use engine_value::Value;
use error::ServerError;
use grafbase_telemetry::graphql::GraphqlResponseStatus;
use graph_entities::QueryResponse;
use http::{
    header::{HeaderMap, HeaderName},
    HeaderValue,
};
use runtime::cache::{CacheMetadata, Cacheable};
use serde::{ser::SerializeMap, Deserialize, Serialize};

pub use cache_control::CacheControl;
pub use streaming::*;
pub mod error;

mod cache_control;
mod streaming;

/// GraphQL operation used in the request.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct GraphqlOperationAnalyticsAttributes {
    pub name: Option<String>,
    pub r#type: common_types::OperationType,
    #[serde(default)]
    pub used_fields: String,
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
    pub graphql_operation: Option<GraphqlOperationAnalyticsAttributes>,
}

impl Response {
    pub fn to_graphql_response(&self) -> GraphQlResponse {
        GraphQlResponse(self)
    }

    /// Create a new successful response with the data.
    #[must_use]
    pub fn new(mut data: QueryResponse, graphql_operation: GraphqlOperationAnalyticsAttributes) -> Self {
        data.shrink_to_fit();
        Self {
            data,
            graphql_operation: Some(graphql_operation),
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
    pub fn bad_request(
        errors: Vec<ServerError>,
        graphql_operation: Option<GraphqlOperationAnalyticsAttributes>,
    ) -> Self {
        Self {
            errors,
            graphql_operation,
            ..Default::default()
        }
    }

    /// Create a response from some errors.
    #[must_use]
    pub fn from_errors_with_type(errors: Vec<ServerError>, operation_type: OperationType) -> Self {
        Self {
            errors,
            graphql_operation: Some(GraphqlOperationAnalyticsAttributes {
                r#type: match operation_type {
                    OperationType::Query => common_types::OperationType::Query {
                        is_introspection: false,
                    },
                    OperationType::Mutation => common_types::OperationType::Mutation,
                    OperationType::Subscription => common_types::OperationType::Subscription,
                },
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    #[must_use]
    pub fn with_graphql_operation(self, graphql_operation: GraphqlOperationAnalyticsAttributes) -> Self {
        Self {
            graphql_operation: Some(graphql_operation),
            ..self
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

    pub fn status(&self) -> GraphqlResponseStatus {
        if self.errors.is_empty() {
            GraphqlResponseStatus::Success
        } else if self.data.root.is_none() {
            GraphqlResponseStatus::RequestError {
                count: self.errors.len() as u64,
            }
        } else {
            GraphqlResponseStatus::FieldError {
                count: self.errors.len() as u64,
                data_is_null: self.data.is_null(),
            }
        }
    }

    pub fn first_error_message(&self) -> Option<String> {
        self.errors.first().map(ToString::to_string)
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
        if self.0.data.root.is_some() {
            map.serialize_entry("data", &self.0.data.as_graphql_data())?;
        }
        if !self.0.errors.is_empty() {
            map.serialize_entry("errors", &self.0.errors)?;
        }
        if !self.0.extensions.is_empty() {
            map.serialize_entry("extensions", &self.0.extensions)?;
        }
        map.end()
    }
}

impl Cacheable for Response {
    fn metadata(&self) -> CacheMetadata {
        CacheMetadata {
            max_age: Duration::from_secs(self.cache_control.max_age as u64),
            stale_while_revalidate: Duration::from_secs(self.cache_control.stale_while_revalidate as u64),
            tags: self.data.cache_tags().iter().cloned().collect::<Vec<_>>(),
            should_purge_related: self
                .graphql_operation
                .as_ref()
                .is_some_and(|operation| operation.r#type == common_types::OperationType::Mutation)
                && !self.data.cache_tags().is_empty(),
            should_cache: !self
                .graphql_operation
                .as_ref()
                .is_some_and(|operation| operation.r#type == common_types::OperationType::Mutation)
                && self.errors.is_empty()
                && self.cache_control.max_age != 0,
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
