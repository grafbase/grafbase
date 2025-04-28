use std::borrow::Cow;

use serde::{Deserializer, de::DeserializeSeed};

use crate::response::{ErrorCode, ErrorPath, GraphqlError, SharedResponsePart};

pub(in crate::resolver::graphql) trait SubgraphToSupergraphErrorPathConverter {
    fn convert(&self, path: serde_json::Value) -> Option<ErrorPath>;
}

impl<F> SubgraphToSupergraphErrorPathConverter for F
where
    F: Fn(serde_json::Value) -> Option<ErrorPath>,
{
    fn convert(&self, path: serde_json::Value) -> Option<ErrorPath> {
        self(path)
    }
}

/// Deserialize the `errors` field of a GraphQL response with the help of a ErrorPathConverter.
pub(in crate::resolver::graphql) struct GraphqlErrorsSeed<'resp, ErrorPathConverter> {
    pub response_part: SharedResponsePart<'resp>,
    pub path_converter: ErrorPathConverter,
}

impl<'resp, ErrorPathConverter> GraphqlErrorsSeed<'resp, ErrorPathConverter>
where
    ErrorPathConverter: SubgraphToSupergraphErrorPathConverter,
{
    pub fn new(response_part: SharedResponsePart<'resp>, path_converter: ErrorPathConverter) -> Self {
        Self {
            response_part,
            path_converter,
        }
    }
}

impl<'resp, 'de, ErrorPathConverter> DeserializeSeed<'de> for GraphqlErrorsSeed<'resp, ErrorPathConverter>
where
    'resp: 'de,
    ErrorPathConverter: SubgraphToSupergraphErrorPathConverter,
{
    type Value = usize;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        let errors = <Option<Vec<SubgraphGraphqlError>> as serde::Deserialize>::deserialize(deserializer)?;

        let Some(errors) = errors else {
            return Ok(0);
        };

        let errors_count = errors.len();
        let errors = errors
            .into_iter()
            .map(|subgraph_error| {
                let mut error = GraphqlError::new(subgraph_error.message, ErrorCode::SubgraphError);
                if let Some(path) = self.path_converter.convert(subgraph_error.path) {
                    error = error.with_path(path);
                }
                if let Some(mut extensions) = subgraph_error.extensions {
                    error.extensions.append(&mut extensions);
                }
                error
            })
            .collect();
        self.response_part.borrow_mut().set_subgraph_errors(errors);
        Ok(errors_count)
    }
}

#[serde_with::serde_as]
#[derive(serde::Deserialize)]
struct SubgraphGraphqlError {
    message: String,
    #[serde(default)]
    path: serde_json::Value,
    #[serde(default)]
    #[serde_as(as = "Option<serde_with::Map<_, _>>")]
    extensions: Option<Vec<(Cow<'static, str>, serde_json::Value)>>,
}
