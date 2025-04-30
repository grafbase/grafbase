use serde::{
    Deserializer,
    de::{DeserializeSeed, IgnoredAny, MapAccess, Visitor},
};

use crate::response::{ErrorPath, ErrorPathSegment};

use super::SubgraphToSupergraphErrorPathConverter;

/// Deserialize the `data` field in the GraphQL response when it's a federated entity request
/// returning `_entities` field and nothing else.
pub(in crate::resolver::graphql) struct EntitiesDataSeed<EntitiesSeed> {
    entities_seed: EntitiesSeed,
}

impl<EntitiesSeed> EntitiesDataSeed<EntitiesSeed> {
    pub fn new(entities_seed: EntitiesSeed) -> Self {
        Self { entities_seed }
    }
}

impl<'de, EntitiesSeed> DeserializeSeed<'de> for EntitiesDataSeed<EntitiesSeed>
where
    EntitiesSeed: DeserializeSeed<'de, Value = ()>,
{
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(self)
    }
}

impl<'de, EntitiesSeed> Visitor<'de> for EntitiesDataSeed<EntitiesSeed>
where
    EntitiesSeed: DeserializeSeed<'de, Value = ()>,
{
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("data with an entities list")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let Self { entities_seed } = self;
        let mut entities_seed = Some(entities_seed);
        while let Some(key) = map.next_key::<EntitiesKey>()? {
            match key {
                EntitiesKey::Entities => {
                    if let Some(seed) = entities_seed.take() {
                        map.next_value_seed(seed)?;
                    }
                }
                EntitiesKey::Unknown => {
                    map.next_value::<IgnoredAny>()?;
                }
            }
        }

        if let Some(seed) = entities_seed {
            seed.deserialize(serde_json::Value::Array(Vec::new()))
                .expect("Deserializer never fails");
        }

        Ok(())
    }
}

#[derive(serde::Deserialize)]
enum EntitiesKey {
    #[serde(rename = "_entities")]
    Entities,
    #[serde(other)]
    Unknown,
}

pub struct EntityErrorPathConverter<F>(pub F);

impl<F> SubgraphToSupergraphErrorPathConverter for EntityErrorPathConverter<F>
where
    F: Fn(usize) -> Option<Vec<ErrorPathSegment>>,
{
    fn convert(&self, path: serde_json::Value) -> Option<ErrorPath> {
        let serde_json::Value::Array(path) = path else {
            return None;
        };
        let mut path = path.into_iter();
        if path.next()?.as_str()? != "_entities" {
            return None;
        }

        let index = path.next()?.as_u64()? as usize;
        let mut out = (self.0)(index)?;

        for segment in path {
            match segment {
                serde_json::Value::String(field) => {
                    out.push(ErrorPathSegment::UnknownField(field.into_boxed_str()));
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
