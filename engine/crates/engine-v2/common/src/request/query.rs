use std::borrow::Cow;

use serde::{de::DeserializeOwned, Deserializer};

use crate::{BatchGraphqlRequest, BorrowedVariables, GraphqlRequest};

pub struct BorrowedQueryRequest<'a, E>(GraphqlRequest<'a, E>);

impl<'a, E> From<BorrowedQueryRequest<'a, E>> for BatchGraphqlRequest<'a, E> {
    fn from(value: BorrowedQueryRequest<'a, E>) -> Self {
        BatchGraphqlRequest::Single(value.0)
    }
}

impl<'a, E> From<BorrowedQueryRequest<'a, E>> for GraphqlRequest<'a, E> {
    fn from(value: BorrowedQueryRequest<'a, E>) -> Self {
        value.0
    }
}

#[derive(serde::Deserialize)]
struct QueryParams<'a> {
    #[serde(default)]
    query: Option<Cow<'a, str>>,
    #[serde(default)]
    doc_id: Option<Cow<'a, str>>,
    #[serde(default)]
    variables: Option<Cow<'a, str>>,
    #[serde(default)]
    operation_name: Option<Cow<'a, str>>,
    #[serde(default)]
    extensions: Option<Cow<'a, str>>,
}

impl<'de, E: Default + DeserializeOwned> serde::Deserialize<'de> for BorrowedQueryRequest<'de, E> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let query_params = QueryParams::<'de>::deserialize(deserializer)?;
        let request = GraphqlRequest {
            query: query_params.query,
            doc_id: query_params.doc_id,
            operation_name: query_params.operation_name,
            variables: query_params
                .variables
                .map(|json| {
                    serde_json::from_str(json.as_ref())
                        .map_err(serde::de::Error::custom)
                        .map(BorrowedVariables::into_owned)
                })
                .transpose()?
                .unwrap_or_default(),
            extensions: query_params
                .extensions
                .map(|json| serde_json::from_str(json.as_ref()).map_err(serde::de::Error::custom))
                .transpose()?
                .unwrap_or_default(),
        };
        Ok(BorrowedQueryRequest(request))
    }
}
