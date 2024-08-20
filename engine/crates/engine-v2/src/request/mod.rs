use engine::{RequestExtensions, Variables};
use serde::Deserializer;

#[derive(serde::Deserialize)]
#[serde(untagged)]
pub(crate) enum BatchRequest {
    Single(Request),
    Batch(Vec<Request>),
}

#[derive(serde::Deserialize, Debug)]
pub(crate) struct Request {
    #[serde(default)]
    pub query: Option<String>,
    #[serde(default, rename = "operationName")]
    pub operation_name: Option<String>,
    #[serde(default)]
    pub doc_id: Option<String>,
    #[serde(default)]
    pub variables: Variables,
    #[serde(default)]
    pub extensions: RequestExtensions,
}

pub(crate) struct QueryParamsRequest(Request);

impl From<QueryParamsRequest> for Request {
    fn from(QueryParamsRequest(request): QueryParamsRequest) -> Request {
        request
    }
}

impl<'de> serde::Deserialize<'de> for QueryParamsRequest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let QueryParams {
            query,
            doc_id,
            variables,
            operation_name,
            extensions,
        } = QueryParams::deserialize(deserializer)?;
        Ok(QueryParamsRequest(Request {
            query,
            operation_name,
            doc_id,
            variables: variables
                .as_deref()
                .map(serde_json::from_str)
                .transpose()
                .map_err(serde::de::Error::custom)?
                .unwrap_or_default(),
            extensions: extensions
                .as_deref()
                .map(serde_json::from_str)
                .transpose()
                .map_err(serde::de::Error::custom)?
                .unwrap_or_default(),
        }))
    }
}

#[derive(serde::Deserialize)]
struct QueryParams {
    #[serde(default)]
    query: Option<String>,
    #[serde(default)]
    doc_id: Option<String>,
    #[serde(default)]
    variables: Option<String>,
    #[serde(default)]
    operation_name: Option<String>,
    #[serde(default)]
    extensions: Option<String>,
}
