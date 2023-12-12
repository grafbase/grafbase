use std::collections::BTreeMap;

use serde::{de::DeserializeSeed, Deserializer};

use crate::response::{GraphqlError, ResponsePath};

#[derive(serde::Deserialize)]
pub(crate) struct UpstreamGraphqlError {
    pub message: String,
    #[serde(default)]
    pub locations: serde_json::Value,
    #[serde(default)]
    pub path: serde_json::Value,
    #[serde(default)]
    pub extensions: serde_json::Value,
}

pub(crate) struct UpstreamGraphqlErrorsSeed<'a> {
    pub path: Option<ResponsePath>,
    pub errors: &'a mut Vec<GraphqlError>,
}

impl<'de, 'a> DeserializeSeed<'de> for UpstreamGraphqlErrorsSeed<'a> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        let errors = <Vec<UpstreamGraphqlError> as serde::Deserialize>::deserialize(deserializer)?;
        for error in errors {
            let mut extensions = BTreeMap::new();
            if !error.locations.is_null() {
                extensions.insert("upstream_locations".to_string(), error.locations);
            }
            if !error.path.is_null() {
                extensions.insert("upstream_path".to_string(), error.path);
            }
            if !error.extensions.is_null() {
                extensions.insert("upstream_extensions".to_string(), error.extensions);
            }
            self.errors.push(GraphqlError {
                message: format!("Upstream error: {}", error.message),
                locations: vec![],
                path: self.path.clone(),
                extensions,
            });
        }
        Ok(())
    }
}
