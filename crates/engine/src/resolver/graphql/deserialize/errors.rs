use std::borrow::Cow;

use serde::{Deserializer, de::DeserializeSeed};

use crate::response::{ErrorCode, ErrorPath, GraphqlError, SeedState};

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
pub(in crate::resolver::graphql) struct GraphqlErrorsSeed<'ctx, 'parent, 'state, ErrorPathConverter> {
    pub state: &'state SeedState<'ctx, 'parent>,
    pub path_converter: ErrorPathConverter,
}

impl<'ctx, 'parent, 'state, ErrorPathConverter> GraphqlErrorsSeed<'ctx, 'parent, 'state, ErrorPathConverter> {
    pub fn new(state: &'state SeedState<'ctx, 'parent>, path_converter: ErrorPathConverter) -> Self
    where
        ErrorPathConverter: SubgraphToSupergraphErrorPathConverter,
    {
        Self { state, path_converter }
    }
}

impl<'de, ErrorPathConverter> DeserializeSeed<'de> for GraphqlErrorsSeed<'_, '_, '_, ErrorPathConverter>
where
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
        let mut part = self.state.response.borrow_mut();
        for subgraph_error in errors {
            let mut error = GraphqlError::new(subgraph_error.message, ErrorCode::SubgraphError);
            if let Some(path) = self.path_converter.convert(subgraph_error.path) {
                error = error.with_path(path);
            }
            if let Some(mut extensions) = subgraph_error.extensions {
                error.extensions.append(&mut extensions);
            }
            part.errors.push(error);
        }
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
