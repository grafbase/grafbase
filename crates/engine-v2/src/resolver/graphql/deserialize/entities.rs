use std::fmt;

use serde::{
    de::{DeserializeSeed, Error, IgnoredAny, MapAccess, SeqAccess, Visitor},
    Deserializer,
};

use crate::{
    execution::ExecutionContext,
    resolver::graphql::CacheEntry,
    response::{ErrorCode, GraphqlError, ResponseKeys, ResponsePath, SubgraphResponseRefMut, UnpackedResponseEdge},
    Runtime,
};

use super::errors::GraphqlErrorsSeed;

pub(in crate::resolver::graphql) struct EntitiesDataSeed<'resp, R: Runtime> {
    pub ctx: ExecutionContext<'resp, R>,
    pub response: SubgraphResponseRefMut<'resp>,
    pub cache_entries: Option<&'resp [CacheEntry]>,
}

impl<'resp, 'de, R: Runtime> DeserializeSeed<'de> for EntitiesDataSeed<'resp, R>
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

impl<'resp, 'de, R: Runtime> Visitor<'de> for EntitiesDataSeed<'resp, R>
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
                        ctx: self.ctx,
                        response_part: &self.response,
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

struct EntitiesSeed<'resp, 'parent, R: Runtime> {
    ctx: ExecutionContext<'resp, R>,
    response_part: &'parent SubgraphResponseRefMut<'resp>,
    cache_entries: Option<std::slice::Iter<'parent, CacheEntry>>,
}

impl<'resp, 'de, 'parent, R: Runtime> DeserializeSeed<'de> for EntitiesSeed<'resp, 'parent, R>
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

impl<'resp, 'de, 'parent, R: Runtime> Visitor<'de> for EntitiesSeed<'resp, 'parent, R>
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
        while let Some(seed) = self.response_part.next_seed(&self.ctx) {
            let maybe_cache_data = self
                .cache_entries
                .as_mut()
                .map(|some| some.next().expect("cache entries to be the correct length"))
                .and_then(CacheEntry::as_data);

            let result = match maybe_cache_data {
                Some(data) => {
                    // The current element was found in the cache
                    seed.deserialize(&mut serde_json::Deserializer::from_slice(data))
                        .map(Some)
                        .map_err(A::Error::custom)
                }
                _ => {
                    // The current element was not found in the cache so should be read from the response
                    seq.next_element_seed(seed)
                }
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

pub(in crate::resolver::graphql) struct EntitiesErrorsSeed<'resp> {
    pub response: SubgraphResponseRefMut<'resp>,
    pub response_keys: &'resp ResponseKeys,
}

impl<'resp> EntitiesErrorsSeed<'resp> {
    pub fn new<R: Runtime>(ctx: ExecutionContext<'resp, R>, response: SubgraphResponseRefMut<'resp>) -> Self {
        Self {
            response,
            response_keys: &ctx.operation.cached.solved.response_keys,
        }
    }
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
