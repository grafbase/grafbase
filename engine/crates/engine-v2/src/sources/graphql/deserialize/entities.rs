use std::fmt;

use serde::{
    de::{DeserializeSeed, IgnoredAny, MapAccess, SeqAccess, Visitor},
    Deserializer,
};

use crate::{
    execution::PlanWalker,
    response::{ErrorCode, GraphqlError, ResponseKeys, ResponsePartMut, ResponsePath, UnpackedResponseEdge},
};

use super::errors::GraphqlErrorsSeed;

pub(in crate::sources::graphql) struct EntitiesDataSeed<'a> {
    pub response_part: &'a ResponsePartMut<'a>,
    pub plan: PlanWalker<'a>,
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
                        response_part: self.response_part,
                        plan: self.plan,
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

struct EntitiesSeed<'a> {
    response_part: &'a ResponsePartMut<'a>,
    plan: PlanWalker<'a>,
}

impl<'de, 'a> DeserializeSeed<'de> for EntitiesSeed<'a> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(self)
    }
}

impl<'de, 'a> Visitor<'de> for EntitiesSeed<'a> {
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a non null entities list")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        while let Some(seed) = self.response_part.next_seed(self.plan) {
            match seq.next_element_seed(seed) {
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

pub(in crate::sources::graphql) struct EntitiesErrorsSeed<'a> {
    pub response_part: &'a ResponsePartMut<'a>,
    pub response_keys: &'a ResponseKeys,
}

impl<'a> GraphqlErrorsSeed<'a> for EntitiesErrorsSeed<'a> {
    fn response_part(&self) -> &'a ResponsePartMut<'a> {
        self.response_part
    }

    fn convert_path(&self, path: &serde_json::Value) -> Option<ResponsePath> {
        let mut path = path.as_array()?.iter();
        if path.next()?.as_str()? != "_entities" {
            return None;
        }

        let mut out = self
            .response_part
            .root_response_object_refs()
            .get(path.next()?.as_u64()? as usize)?
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
