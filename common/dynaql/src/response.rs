use std::collections::BTreeMap;

use graph_entities::QueryResponse;
use http::header::{HeaderMap, HeaderName};
use http::HeaderValue;
use serde::Serialize;

use crate::{CacheControl, Result, ServerError, Value};

/// Query response
#[derive(Debug, Default, Serialize)]
pub struct Response {
    /// Data of query result
    #[serde(default)]
    pub data: QueryResponse,

    /// Extensions result
    #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
    pub extensions: BTreeMap<String, Value>,

    /// Cache control value
    #[serde(skip)]
    pub cache_control: CacheControl,

    /// Errors
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub errors: Vec<ServerError>,

    /// HTTP headers
    #[serde(skip)]
    pub http_headers: HeaderMap,
}

impl Response {
    pub fn to_json_value(&self) -> serde_json::Result<serde_json::Value> {
        let mut fields = serde_json::Map::new();
        fields.insert("data".to_string(), self.data.to_json_value()?);
        if !self.errors.is_empty() {
            fields.insert("errors".to_string(), serde_json::to_value(&self.errors)?);
        }
        if !self.extensions.is_empty() {
            fields.insert(
                "extensions".to_string(),
                serde_json::to_value(&self.extensions)?,
            );
        }
        Ok(serde_json::Value::Object(fields))
    }

    /// Create a new successful response with the data.
    #[must_use]
    pub fn new(data: QueryResponse) -> Self {
        Self {
            data,
            ..Default::default()
        }
    }

    /// Create a response from some errors.
    #[must_use]
    pub fn from_errors(errors: Vec<ServerError>) -> Self {
        Self {
            errors,
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
        Self {
            http_headers,
            ..self
        }
    }

    /// Set the cache control of the response.
    #[must_use]
    pub fn cache_control(self, cache_control: CacheControl) -> Self {
        Self {
            cache_control,
            ..self
        }
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
#[derive(Debug, Serialize)]
#[serde(untagged)]
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
            BatchResponse::Single(resp) => resp.cache_control,
            BatchResponse::Batch(resp) => resp.iter().fold(CacheControl::default(), |acc, item| {
                acc.merge(&item.cache_control)
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
            current_name
                .clone()
                .map(|current_name| (current_name, value))
        })
    }

    pub fn to_json_value(&self) -> serde_json::Result<serde_json::Value> {
        match self {
            Self::Batch(multiple) => {
                let mut out = Vec::new();
                for resp in multiple {
                    out.push(resp.to_json_value()?);
                }
                Ok(serde_json::Value::Array(out))
            }
            Self::Single(single) => single.to_json_value(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_response_single() {
        let json = Value::Boolean(true).into_json().unwrap();
        let mut resp = QueryResponse::default();
        let id = resp.from_serde_value(json);
        resp.set_root_unchecked(id);

        let resp = Response::new(resp);

        let resp = BatchResponse::Single(resp);
        assert_eq!(
            resp.to_json_value().unwrap().to_string(),
            r#"{"data":true}"#
        );
    }

    #[test]
    fn test_batch_response_batch() {
        let json1 = Value::Boolean(true).into_json().unwrap();
        let mut resp1 = QueryResponse::default();
        let id = resp1.from_serde_value(json1);
        resp1.set_root_unchecked(id);
        let resp1 = Response::new(resp1);

        let json2 = Value::String("1".to_string()).into_json().unwrap();
        let mut resp2 = QueryResponse::default();
        let id = resp2.from_serde_value(json2);
        resp2.set_root_unchecked(id);
        let resp2 = Response::new(resp2);

        let resp = BatchResponse::Batch(vec![resp1, resp2]);
        assert_eq!(
            resp.to_json_value().unwrap().to_string(),
            r#"[{"data":true},{"data":"1"}]"#
        );
    }
}
