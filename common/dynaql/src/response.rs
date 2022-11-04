use std::collections::BTreeMap;
use std::fmt::{self, Write};

use graph_entities::QueryResponse;
use http::header::{HeaderMap, HeaderName};
use http::HeaderValue;
use serde::{Deserialize, Serialize};

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
    pub fn to_response_string(&self) -> String {
        let errors = if !self.errors.is_empty() {
            format!(
                ",\"errors\":{}",
                serde_json::to_string(&self.errors).expect("Unchecked")
            )
        } else {
            String::new()
        };

        let extensions = if !self.extensions.is_empty() {
            format!(
                ",\"extensions\":{}",
                serde_json::to_string(&self.extensions).expect("Unchecked")
            )
        } else {
            String::new()
        };

        format!("{{\"data\":{}{errors}{extensions}}}", self.data.to_string())
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

    pub fn to_json(&self, f: &mut dyn Write) -> fmt::Result {
        match self {
            Self::Batch(multiple) => {
                write!(f, "[")?;
                let len = multiple.len();
                for (i, one) in multiple.into_iter().enumerate() {
                    write!(f, "{}", one.to_response_string())?;
                    if i != (len - 1) {
                        write!(f, ",")?;
                    }
                }
                write!(f, "]")?;
                Ok(())
            }
            Self::Single(single) => write!(f, "{}", single.to_response_string()),
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
        let mut output = String::new();
        resp.to_json(&mut output);

        assert_eq!(output, r#"{"data":true}"#);
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
        let mut output = String::new();
        resp.to_json(&mut output);
        assert_eq!(output, r#"[{"data":true},{"data":"1"}]"#);
    }
}
