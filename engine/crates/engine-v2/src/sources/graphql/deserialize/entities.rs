use std::fmt;

use serde::{
    de::{DeserializeSeed, Error, IgnoredAny, MapAccess, SeqAccess, Visitor},
    Deserializer,
};

use crate::{
    execution::PlanWalker,
    response::{ErrorCode, GraphqlError, ResponseKeys, ResponsePath, SubgraphResponseRefMut, UnpackedResponseEdge},
    sources::graphql::CachedEntity,
};

use super::errors::GraphqlErrorsSeed;

pub(in crate::sources::graphql) struct EntitiesDataSeed<'resp> {
    pub response: SubgraphResponseRefMut<'resp>,
    pub plan: PlanWalker<'resp>,
    pub cache_entries: Option<&'resp [CachedEntity]>,
}

impl<'resp, 'de> DeserializeSeed<'de> for EntitiesDataSeed<'resp>
where
    'resp: 'de,
{
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(self)
    }
}

impl<'resp, 'de> Visitor<'de> for EntitiesDataSeed<'resp>
where
    'resp: 'de,
{
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
                        response_part: &self.response,
                        plan: self.plan,
                        cache_entries: self.cache_entries.map(|slice| slice.iter()),
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

struct EntitiesSeed<'resp, 'parent> {
    response_part: &'parent SubgraphResponseRefMut<'resp>,
    plan: PlanWalker<'resp>,
    cache_entries: Option<std::slice::Iter<'parent, CachedEntity>>,
}

impl<'resp, 'de, 'parent> DeserializeSeed<'de> for EntitiesSeed<'resp, 'parent>
where
    'resp: 'de,
{
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(self)
    }
}

impl<'resp, 'de, 'parent> Visitor<'de> for EntitiesSeed<'resp, 'parent>
where
    'resp: 'de,
{
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a non null entities list")
    }

    fn visit_seq<A>(mut self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        while let Some(seed) = self.response_part.next_seed(self.plan) {
            let next_cache_entry = self.cache_entries.as_mut().and_then(Iterator::next);
            let result = match next_cache_entry {
                Some(entry) if entry.data.is_some() => seed
                    .deserialize(&mut serde_json::Deserializer::from_slice(
                        entry.data.as_deref().unwrap(),
                    ))
                    .map(Some)
                    .map_err(A::Error::custom),
                _ => seq.next_element_seed(seed),
            };

            match result {
                Ok(Some(_)) => continue,
                Ok(None) => break,
                Err(err) => {
                    // Discarding the rest of the list
                    while seq.next_element::<IgnoredAny>().unwrap_or_default().is_some() {}
                    return Err(err);
                }
            }
        }
        if seq.next_element::<IgnoredAny>()?.is_some() {
            self.response_part.push_error(GraphqlError::new(
                "Received more entities than expected",
                ErrorCode::SubgraphInvalidResponseError,
            ));
            while seq.next_element::<IgnoredAny>()?.is_some() {}
        }
        Ok(())
    }
}

pub(in crate::sources::graphql) struct EntitiesErrorsSeed<'resp> {
    pub response: SubgraphResponseRefMut<'resp>,
    pub response_keys: &'resp ResponseKeys,
}

impl<'resp> GraphqlErrorsSeed<'resp> for EntitiesErrorsSeed<'resp> {
    fn response(&self) -> &SubgraphResponseRefMut<'resp> {
        &self.response
    }

    fn convert_path(&self, path: &serde_json::Value) -> Option<ResponsePath> {
        let mut path = path.as_array()?.iter();
        if path.next()?.as_str()? != "_entities" {
            return None;
        }

        let mut out = self
            .response
            .get_root_response_object(path.next()?.as_u64()? as usize)?
            .path
            .clone();

        for edge in path {
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
