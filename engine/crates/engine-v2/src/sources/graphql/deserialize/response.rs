use std::fmt;

use grafbase_telemetry::gql_response_status::GraphqlResponseStatus;
use serde::{
    de::{DeserializeSeed, IgnoredAny, MapAccess, Visitor},
    Deserializer,
};

use super::errors::{ConcreteGraphqlErrorsSeed, GraphqlErrorsSeed};

pub(in crate::sources::graphql) struct GraphqlResponseSeed<DataSeed, ErrorSeed> {
    data_seed: Option<DataSeed>,
    errors_seed: Option<ConcreteGraphqlErrorsSeed<ErrorSeed>>,
}

impl<DataSeed, ErrorSeed> GraphqlResponseSeed<DataSeed, ErrorSeed> {
    pub fn new(data_seed: DataSeed, errors_seed: ErrorSeed) -> Self {
        Self {
            data_seed: Some(data_seed),
            errors_seed: Some(ConcreteGraphqlErrorsSeed(errors_seed)),
        }
    }
}

impl<'de, DataSeed, ErrorsSeed> DeserializeSeed<'de> for GraphqlResponseSeed<DataSeed, ErrorsSeed>
where
    DataSeed: DeserializeSeed<'de, Value = ()>,
    ErrorsSeed: GraphqlErrorsSeed<'de>,
{
    type Value = GraphqlResponseStatus;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(self)
    }
}

impl<'de, DataSeed, ErrorsSeed> Visitor<'de> for GraphqlResponseSeed<DataSeed, ErrorsSeed>
where
    DataSeed: DeserializeSeed<'de, Value = ()>,
    ErrorsSeed: GraphqlErrorsSeed<'de>,
{
    type Value = GraphqlResponseStatus;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a valid GraphQL response")
    }

    fn visit_map<A>(mut self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut data_is_null_result = Ok(true);
        let mut errors_count = 0;
        while let Some(key) = map.next_key::<ResponseKey>()? {
            match key {
                ResponseKey::Data => {
                    if let Some(seed) = self.data_seed.take() {
                        data_is_null_result = map.next_value_seed(NullableDataSeed { seed });
                    }
                }
                ResponseKey::Errors => {
                    if let Some(seed) = self.errors_seed.take() {
                        errors_count = map.next_value_seed(seed)?;
                    }
                }
                ResponseKey::Unknown => {
                    map.next_value::<IgnoredAny>()?;
                }
            };
        }

        let data_is_present = self.data_seed.is_some();
        let status = if errors_count == 0 {
            GraphqlResponseStatus::Success
        } else if data_is_present {
            GraphqlResponseStatus::FieldError {
                count: errors_count as u64,
                data_is_null: data_is_null_result?,
            }
        } else {
            GraphqlResponseStatus::RequestError {
                count: errors_count as u64,
            }
        };

        Ok(status)
    }
}

struct NullableDataSeed<Seed> {
    seed: Seed,
}

impl<'de, Seed> DeserializeSeed<'de> for NullableDataSeed<Seed>
where
    Seed: DeserializeSeed<'de, Value = ()>,
{
    type Value = bool;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_option(self)
    }
}

impl<'de, Seed> Visitor<'de> for NullableDataSeed<Seed>
where
    Seed: DeserializeSeed<'de, Value = ()>,
{
    type Value = bool;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a nullable value")
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_none()
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(true)
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        self.seed.deserialize(deserializer)?;
        Ok(false)
    }
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "lowercase")]
enum ResponseKey {
    Data,
    Errors,
    #[serde(other)]
    Unknown,
}
