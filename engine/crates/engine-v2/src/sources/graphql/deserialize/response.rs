use std::fmt;

use serde::{
    de::{DeserializeSeed, MapAccess, Visitor},
    Deserializer,
};

use super::UpstreamGraphqlErrorsSeed;
use crate::response::{GraphqlError, ResponsePath};

pub(crate) struct GraphqlResponseSeed<DataSeed> {
    data: Option<DataSeed>,
    err_path: Option<ResponsePath>,
}

impl<D> GraphqlResponseSeed<D> {
    pub fn new(data: D, err_path: Option<ResponsePath>) -> Self {
        Self {
            err_path,
            data: Some(data),
        }
    }
}

impl<'de, DataSeed> DeserializeSeed<'de> for GraphqlResponseSeed<DataSeed>
where
    DataSeed: DeserializeSeed<'de, Value = ()>,
{
    type Value = Vec<GraphqlError>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(self)
    }
}

impl<'de, DataSeed> Visitor<'de> for GraphqlResponseSeed<DataSeed>
where
    DataSeed: DeserializeSeed<'de, Value = ()>,
{
    type Value = Vec<GraphqlError>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a valid GraphQL response")
    }

    fn visit_map<A>(mut self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut serde_errors = vec![];
        let mut upstream_errors = vec![];
        while let Some(key) = map.next_key::<&str>()? {
            let error = match key {
                "data" => match self.data.take() {
                    Some(data) => map.next_value_seed(data).err(),
                    None => Some(serde::de::Error::custom("data key present multiple times.")),
                },
                "errors" => map
                    .next_value_seed(UpstreamGraphqlErrorsSeed {
                        path: self.err_path.clone(),
                        errors: &mut upstream_errors,
                    })
                    .err(),
                _ => map.next_value::<serde::de::IgnoredAny>().err(),
            };
            if let Some(err) = error {
                serde_errors.push(GraphqlError {
                    message: format!("Deserialization failure: {err}"),
                    path: self.err_path.clone(),
                    ..Default::default()
                })
            }
        }
        // If there any upstream errors, no need to show serde errors for this visitor.
        if !upstream_errors.is_empty() {
            Ok(upstream_errors)
        } else {
            Ok(serde_errors)
        }
    }
}
