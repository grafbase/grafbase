use std::{borrow::Cow, collections::VecDeque, fmt};

use schema::ObjectId;
use serde::de::{DeserializeSeed, IgnoredAny, MapAccess, Visitor};

use crate::{
    plan::ConditionalSelectionSetId,
    request::SelectionSetType,
    response::{
        write::deserialize::{key::Key, SeedContextInner},
        ResponseValue,
    },
};

use super::{CollectedSelectionSetSeed, ObjectIdentifier};

pub(crate) struct ConditionalSelectionSetSeed<'ctx, 'parent> {
    pub ctx: &'parent SeedContextInner<'ctx>,
    pub selection_set_ty: SelectionSetType,
    pub selection_set_ids: Cow<'parent, [ConditionalSelectionSetId]>,
}

impl<'de, 'ctx, 'parent> DeserializeSeed<'de> for ConditionalSelectionSetSeed<'ctx, 'parent> {
    type Value = ResponseValue;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(self)
    }
}

impl<'de, 'ctx, 'parent> Visitor<'de> for ConditionalSelectionSetSeed<'ctx, 'parent> {
    type Value = ResponseValue;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("an object")
    }

    // later we could also support visit_struct by using the schema as the reference structure.
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        // Ideally we should never have an ProvisionalSelectionSet with a known object id, it
        // means we could have collected fields earlier. But it can happen when a parent had
        // complex type conditions for which we couldn't collect fields.
        if let SelectionSetType::Object(object_id) = self.selection_set_ty {
            return self.deserialize_concrete_object(object_id, map);
        }
        let mut identifier = ObjectIdentifier::new(self.ctx, self.selection_set_ty);
        let mut content = VecDeque::<(_, serde_value::Value)>::new();
        while let Some(key) = map.next_key::<Key<'de>>()? {
            if identifier.discriminant_key_matches(key.as_ref()) {
                return match identifier.determine_object_id_from_discriminant(map.next_value()?) {
                    Some(object_id) => self.deserialize_concrete_object(
                        object_id,
                        ChainedMapAcces {
                            before: content,
                            next_value: None,
                            after: map,
                        },
                    ),
                    _ => {
                        // Discarding the rest of the data.
                        while map.next_entry::<IgnoredAny, IgnoredAny>().unwrap_or_default().is_some() {}
                        return Err(serde::de::Error::custom("Couldn't determine the object type"));
                    }
                };
            }
            // keeping the fields until we find the actual type discriminant.
            content.push_back((key, map.next_value()?));
        }
        Err(serde::de::Error::custom("Couldn't determine the object type"))
    }
}

impl<'ctx, 'parent> ConditionalSelectionSetSeed<'ctx, 'parent> {
    fn deserialize_concrete_object<'de, A>(self, object_id: ObjectId, map: A) -> Result<ResponseValue, A::Error>
    where
        A: MapAccess<'de>,
    {
        let selection_set = self.ctx.plan.collect_fields(object_id, &self.selection_set_ids);
        CollectedSelectionSetSeed::new(self.ctx, &selection_set).visit_map(map)
    }
}

struct ChainedMapAcces<'de, A> {
    before: VecDeque<(Key<'de>, serde_value::Value)>,
    next_value: Option<serde_value::Value>,
    after: A,
}

impl<'de, A> MapAccess<'de> for ChainedMapAcces<'de, A>
where
    A: MapAccess<'de>,
{
    type Error = A::Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        if let Some((key, value)) = self.before.pop_front() {
            self.next_value = Some(value);
            seed.deserialize(serde_value::ValueDeserializer::new(serde_value::Value::String(
                key.into_string(),
            )))
            .map(Some)
        } else {
            self.after.next_key_seed(seed)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        if let Some(value) = self.next_value.take() {
            seed.deserialize(serde_value::ValueDeserializer::new(value))
        } else {
            self.after.next_value_seed(seed)
        }
    }
}
