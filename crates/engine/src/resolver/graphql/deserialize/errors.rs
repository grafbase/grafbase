use serde::{de::DeserializeSeed, Deserializer};

use crate::response::{ErrorCode, ErrorPath, ErrorPathSegment, GraphqlError, SubgraphResponseRefMut};

pub(super) trait GraphqlErrorsSeed<'resp> {
    fn response(&self) -> &SubgraphResponseRefMut<'resp>;
    fn convert_path(&self, path: serde_json::Value) -> Option<ErrorPath>;
}

pub(in crate::resolver::graphql) struct RootGraphqlErrors<'resp> {
    response: SubgraphResponseRefMut<'resp>,
}

impl<'resp> RootGraphqlErrors<'resp> {
    pub fn new(response: SubgraphResponseRefMut<'resp>) -> Self {
        Self { response }
    }
}

impl<'resp> GraphqlErrorsSeed<'resp> for RootGraphqlErrors<'resp> {
    fn response(&self) -> &SubgraphResponseRefMut<'resp> {
        &self.response
    }

    fn convert_path(&self, path: serde_json::Value) -> Option<ErrorPath> {
        let mut out = Vec::new();
        let serde_json::Value::Array(path) = path else {
            return None;
        };
        for segment in path {
            match segment {
                serde_json::Value::String(field) => {
                    out.push(ErrorPathSegment::UnknownField(field));
                }
                serde_json::Value::Number(index) => {
                    out.push(ErrorPathSegment::Index(index.as_u64()? as usize));
                }
                _ => {
                    return None;
                }
            }
        }
        Some(out.into())
    }
}

#[derive(serde::Deserialize)]
pub(super) struct SubgraphGraphqlError {
    pub message: String,
    #[serde(default)]
    pub path: serde_json::Value,
    #[serde(default)]
    pub extensions: serde_json::Value,
}

pub(super) struct ConcreteGraphqlErrorsSeed<T>(pub(super) T);

impl<'resp, 'de, T> DeserializeSeed<'de> for ConcreteGraphqlErrorsSeed<T>
where
    T: GraphqlErrorsSeed<'resp>,
    'resp: 'de,
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
                if let Some(path) = self.0.convert_path(subgraph_error.path) {
                    error = error.with_path(path);
                }
                if !subgraph_error.extensions.is_null() {
                    error = error.with_extension("upstream_extensions", subgraph_error.extensions);
                }
                error
            })
            .collect();
        self.0.response().set_subgraph_errors(errors);
        Ok(errors_count)
    }
}
