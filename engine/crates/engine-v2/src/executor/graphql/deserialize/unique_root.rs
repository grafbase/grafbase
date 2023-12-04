use std::{collections::HashMap, fmt};

use serde::{
    de::{DeserializeSeed, MapAccess, Visitor},
    Deserializer,
};

use super::UpstreamGraphqlError;
use crate::{
    execution::ExecutionContext,
    response::{GraphqlError, ResponseObjectRoot, ResponsePartBuilder},
};

pub struct UniqueRootSeed<'ctx, 'parent> {
    pub ctx: &'parent ExecutionContext<'ctx, 'ctx>,
    pub output: &'ctx mut ResponsePartBuilder,
    pub root: &'parent ResponseObjectRoot,
}

impl<'de, 'ctx, 'parent> DeserializeSeed<'de> for UniqueRootSeed<'ctx, 'parent> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(self)
    }
}

impl<'de, 'ctx, 'parent> Visitor<'de> for UniqueRootSeed<'ctx, 'parent> {
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a valid GraphQL response")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        while let Some(key) = map.next_key::<&str>()? {
            match key {
                "data" => {
                    let writer = self.ctx.writer(self.output, self.root.clone());
                    map.next_value_seed(writer)?;
                }
                "errors" => {
                    let errors = map.next_value::<Vec<UpstreamGraphqlError>>()?;
                    for error in errors {
                        let mut extensions = HashMap::new();
                        if !error.locations.is_null() {
                            extensions.insert("upstream_locations".to_string(), error.locations);
                        }
                        if !error.path.is_null() {
                            extensions.insert("upstream_path".to_string(), error.path);
                        }
                        if !error.extensions.is_null() {
                            extensions.insert("upstream_extensions".to_string(), error.extensions);
                        }
                        self.output.push_error(GraphqlError {
                            message: format!("Upstream error: {}", error.message),
                            locations: vec![],
                            path: Some(self.root.path.clone()),
                            extensions,
                        });
                    }
                }
                _ => {
                    // Discarding data.
                    map.next_value::<serde::de::IgnoredAny>()?;
                }
            };
        }
        Ok(())
    }
}
