use std::fmt;

use serde::{
    de::{DeserializeSeed, IgnoredAny, MapAccess, SeqAccess, Visitor},
    Deserializer,
};

use crate::{
    response::{ResponseBoundaryItem, SeedContext},
    sources::ExecutionError,
};

pub(in crate::sources::graphql) struct EntitiesDataSeed<'a> {
    pub ctx: SeedContext<'a>,
    pub response_boundary: &'a Vec<ResponseBoundaryItem>,
}

impl<'de, 'a> DeserializeSeed<'de> for EntitiesDataSeed<'a> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(self)
    }
}

impl<'de, 'a> Visitor<'de> for EntitiesDataSeed<'a> {
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("data with an entities list")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        while let Some(key) = map.next_key::<EntitiesKey>()? {
            match key {
                EntitiesKey::Entities => {
                    map.next_value_seed(EntitiesSeed {
                        ctx: &self.ctx,
                        response_boundary: self.response_boundary,
                    })?;
                }
                EntitiesKey::Unknown => {
                    map.next_value::<IgnoredAny>()?;
                }
            }
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

struct EntitiesSeed<'ctx, 'parent> {
    ctx: &'parent SeedContext<'ctx>,
    response_boundary: &'ctx Vec<ResponseBoundaryItem>,
}

impl<'de, 'ctx, 'parent> DeserializeSeed<'de> for EntitiesSeed<'ctx, 'parent> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(self)
    }
}

impl<'de, 'ctx, 'parent> Visitor<'de> for EntitiesSeed<'ctx, 'parent> {
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a non null entities list")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut index = 0;
        loop {
            if index >= self.response_boundary.len() {
                if seq.next_element::<IgnoredAny>()?.is_some() {
                    self.ctx.borrow_mut_response_part().push_error(ExecutionError::Internal(
                        "Received more entities than expected".to_string(),
                    ));
                    while seq.next_element::<IgnoredAny>()?.is_some() {}
                }
                break;
            }
            let seed = self.ctx.create_root_seed(&self.response_boundary[index]);
            match seq.next_element_seed(seed) {
                Ok(Some(_)) => {
                    index += 1;
                }
                Ok(None) => break,
                Err(err) => {
                    // Discarding the rest of the list
                    while seq.next_element::<IgnoredAny>().unwrap_or_default().is_some() {}
                    return Err(err);
                }
            }
        }
        Ok(())
    }
}
