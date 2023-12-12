use std::fmt;

use serde::{
    de::{DeserializeSeed, IgnoredAny, MapAccess, Visitor},
    Deserializer,
};

use super::UpstreamGraphqlErrorsSeed;
use crate::response::{GraphqlError, ResponsePath};

pub(crate) struct GraphqlResponseSeed<'errors, DataSeed> {
    data: Option<DataSeed>,
    err_path: Option<ResponsePath>,
    errors: &'errors mut Vec<GraphqlError>,
}

impl<'a, D> GraphqlResponseSeed<'a, D> {
    pub fn new(err_path: Option<ResponsePath>, errors: &'a mut Vec<GraphqlError>, data: D) -> Self {
        Self {
            err_path,
            errors,
            data: Some(data),
        }
    }
}

impl<'de, 'errors, DataSeed> DeserializeSeed<'de> for GraphqlResponseSeed<'errors, DataSeed>
where
    DataSeed: DeserializeSeed<'de, Value = ()>,
{
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(self)
    }
}

impl<'de, 'errors, DataSeed> Visitor<'de> for GraphqlResponseSeed<'errors, DataSeed>
where
    DataSeed: DeserializeSeed<'de, Value = ()>,
{
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a valid GraphQL response")
    }

    fn visit_map<A>(mut self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        while let Some(key) = map.next_key::<&str>()? {
            match key {
                "data" => match self.data.take() {
                    Some(data) => map.next_value_seed(data)?,
                    None => return Err(serde::de::Error::custom("data key present multiple times.")),
                },
                "errors" => map.next_value_seed(UpstreamGraphqlErrorsSeed {
                    path: self.err_path.clone(),
                    errors: self.errors,
                })?,
                _ => {
                    map.next_value::<IgnoredAny>()?;
                }
            }
        }
        Ok(())
    }
}
