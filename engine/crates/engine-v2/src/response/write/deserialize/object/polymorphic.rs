use std::{collections::VecDeque, fmt};

use schema::ObjectId;
use serde::de::{DeserializeSeed, IgnoredAny, MapAccess, Visitor};

use crate::response::{
    write::deserialize::{key::Key, SeedContext},
    ConcreteObjectShapeId, PolymorphicObjectShapeId, ResponseObject, ResponseValue,
};

use super::concrete::ConcreteObjectSeed;

pub(crate) struct PolymorphicObjectSeed<'ctx, 'seed> {
    ctx: &'seed SeedContext<'ctx>,
    possibilities: &'ctx [(ObjectId, ConcreteObjectShapeId)],
}

impl<'ctx, 'seed> PolymorphicObjectSeed<'ctx, 'seed> {
    pub fn new(ctx: &'seed SeedContext<'ctx>, shape_id: PolymorphicObjectShapeId) -> Self {
        let polymorphic = &ctx.operation.response_blueprint[shape_id];
        Self {
            ctx,
            possibilities: &polymorphic.possibilities,
        }
    }
}

impl<'de, 'ctx, 'parent> DeserializeSeed<'de> for PolymorphicObjectSeed<'ctx, 'parent> {
    type Value = ResponseValue;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(self)
    }
}

impl<'de, 'ctx, 'parent> Visitor<'de> for PolymorphicObjectSeed<'ctx, 'parent> {
    type Value = ResponseValue;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("an object")
    }

    // later we could also support visit_struct by using the schema as the reference structure.
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let schema = self.ctx.schema;
        let mut content = VecDeque::<(_, serde_value::Value)>::new();
        while let Some(key) = map.next_key::<Key<'de>>()? {
            if key.as_ref() == "__typename" {
                let value = map.next_value::<Key<'_>>()?;
                let typename = value.as_ref();
                if let Ok(i) = self
                    .possibilities
                    .binary_search_by(|(id, _)| schema[schema[*id].name].as_str().cmp(typename))
                {
                    let (object_id, shape_id) = self.possibilities[i];
                    return ConcreteObjectSeed::new_with_object_id(self.ctx, shape_id, object_id).visit_map(
                        ChainedMapAcces {
                            before: content,
                            next_value: None,
                            after: map,
                        },
                    );
                }

                // Discarding the rest of the data if it does not match any concrete shape
                while map.next_entry::<IgnoredAny, IgnoredAny>()?.is_some() {}

                // Adding empty object instead
                return Ok(self
                    .ctx
                    .writer
                    .push_object(ResponseObject::new(Default::default()))
                    .into());
            }
            // keeping the fields until we find the actual __typename.
            content.push_back((key, map.next_value()?));
        }
        Err(serde::de::Error::custom(
            "Missing __typename. Couldn't determine the object type",
        ))
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
