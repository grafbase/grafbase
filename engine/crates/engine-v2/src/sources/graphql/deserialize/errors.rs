use std::collections::BTreeMap;

use serde::{de::DeserializeSeed, Deserializer};

use crate::response::{GraphqlError, ResponseKeys, ResponsePartMut, ResponsePath, UnpackedResponseEdge};

pub(super) trait GraphqlErrorsSeed<'a> {
    fn response_part(&self) -> &ResponsePartMut<'a>;
    fn convert_path(&self, path: &serde_json::Value) -> Option<ResponsePath>;
}

pub(in crate::sources::graphql) struct RootGraphqlErrors<'a> {
    pub response_part: &'a ResponsePartMut<'a>,
    pub response_keys: &'a ResponseKeys,
}

impl<'a> GraphqlErrorsSeed<'a> for RootGraphqlErrors<'a> {
    fn response_part(&self) -> &ResponsePartMut<'a> {
        self.response_part
    }

    fn convert_path(&self, path: &serde_json::Value) -> Option<ResponsePath> {
        let mut out = ResponsePath::default();
        for edge in path.as_array()? {
            if let Some(index) = edge.as_u64() {
                out.push(index as usize);
            } else {
                let key = edge.as_str()?;
                let response_key = self.response_keys.get(key)?;
                // We need this path for two reasons only:
                // - To report nicely in the error message
                // - To know whether an error exist if we're missing the appropriate data for a
                //   response object.
                // For the latter we only check whether there is an error at all, not if it's one
                // that could actually propagate up to the root response object. That's a lot more
                // work and very likely useless.
                // So, currently, we'll never read those fields and treat them as extra field
                // to cram them into an ResponseEdge.
                out.push(UnpackedResponseEdge::ExtraFieldResponseKey(response_key.into()))
            }
        }
        Some(out)
    }
}

#[derive(serde::Deserialize)]
pub(super) struct SubgraphGraphqlError {
    pub message: String,
    #[serde(default)]
    pub locations: serde_json::Value,
    #[serde(default)]
    pub path: serde_json::Value,
    #[serde(default)]
    pub extensions: serde_json::Value,
}

pub(super) struct ConcreteGraphqlErrorsSeed<T>(pub(super) T);

impl<'de, T> DeserializeSeed<'de> for ConcreteGraphqlErrorsSeed<T>
where
    T: GraphqlErrorsSeed<'de>,
{
    type Value = usize;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        let errors = <Vec<SubgraphGraphqlError> as serde::Deserialize>::deserialize(deserializer)?;
        let errors_count = errors.len();
        let errors = errors
            .into_iter()
            .map(|error| {
                let mut extensions = BTreeMap::new();
                if !error.locations.is_null() {
                    extensions.insert("upstream_locations".to_string(), error.locations);
                }
                let path = self.0.convert_path(&error.path);
                if path.is_none() && !error.path.is_null() {
                    extensions.insert("upstream_path".to_string(), error.path);
                }
                if !error.extensions.is_null() {
                    extensions.insert("upstream_extensions".to_string(), error.extensions);
                }
                GraphqlError {
                    message: format!("Upstream error: {}", error.message),
                    path,
                    extensions,
                    ..Default::default()
                }
            })
            .collect();
        self.0.response_part().push_errors(errors);
        Ok(errors_count)
    }
}
