use serde::{
    Deserializer,
    de::{DeserializeSeed, EnumAccess, IgnoredAny, MapAccess, SeqAccess, Visitor},
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

impl<'de, EntitiesSeed> EntitiesDataSeed<EntitiesSeed>
where
    EntitiesSeed: DeserializeSeed<'de, Value = ()>,
{
    fn unexpected_type<E>(self) -> Result<(), E>
    where
        E: serde::de::Error,
    {
        self.entities_seed
            .deserialize(serde_json::Value::Array(Vec::new()))
            .expect("Deserializer never fails");

        Ok(())
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
        deserializer.deserialize_any(self)
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

    fn visit_bool<E>(self, _v: bool) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.unexpected_type()
    }

    fn visit_i64<E>(self, _v: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.unexpected_type()
    }

    fn visit_i128<E>(self, _v: i128) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.unexpected_type()
    }

    fn visit_u64<E>(self, _v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.unexpected_type()
    }

    fn visit_u128<E>(self, _v: u128) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.unexpected_type()
    }

    fn visit_f64<E>(self, _v: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.unexpected_type()
    }

    fn visit_char<E>(self, _v: char) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.unexpected_type()
    }

    fn visit_str<E>(self, _v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.unexpected_type()
    }

    fn visit_borrowed_str<E>(self, _v: &'de str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.unexpected_type()
    }

    fn visit_string<E>(self, _v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.unexpected_type()
    }

    fn visit_bytes<E>(self, _v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.unexpected_type()
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.unexpected_type()
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.unexpected_type()
    }

    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        while seq.next_element::<IgnoredAny>()?.is_some() {}
        self.unexpected_type()
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

    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    where
        A: EnumAccess<'de>,
    {
        let _ = data.variant::<IgnoredAny>()?;
        self.unexpected_type()
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
    F: Fn(usize) -> Option<ErrorPath>,
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
        Some(out)
    }
}
