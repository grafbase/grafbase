use serde::de::{DeserializeSeed, IgnoredAny, Visitor};

use crate::response::Key;

pub(in crate::resolver) struct NestedSeed<'ctx, Seed> {
    pub key: &'ctx str,
    pub seed: Seed,
}

impl<'de, 'ctx, Seed> DeserializeSeed<'de> for NestedSeed<'ctx, Seed>
where
    'ctx: 'de,
    Seed: DeserializeSeed<'de>,
{
    type Value = <Seed as DeserializeSeed<'de>>::Value;
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }
}

impl<'de, 'ctx, Seed> Visitor<'de> for NestedSeed<'ctx, Seed>
where
    'ctx: 'de,
    Seed: DeserializeSeed<'de>,
{
    type Value = <Seed as DeserializeSeed<'de>>::Value;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("an object")
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
        Ok(self
            .seed
            .deserialize(serde_json::Value::Null)
            .expect("Deserializer never fails"))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let Self {
            key: expected_key,
            seed,
        } = self;
        enum State<S, V> {
            Seed(S),
            Value(V),
        }
        let mut state = State::Seed(seed);

        while let Some(key) = map.next_key::<Key<'_>>()? {
            if key.as_ref() == expected_key {
                state = match state {
                    State::Seed(seed) => State::Value(map.next_value_seed(seed)?),
                    s => {
                        map.next_value::<IgnoredAny>()?;
                        s
                    }
                };
            } else {
                map.next_value::<IgnoredAny>()?;
            }
        }

        match state {
            State::Value(value) => Ok(value),
            State::Seed(seed) => Ok(seed
                .deserialize(serde_json::Value::Null)
                .expect("Deserializer never fails")),
        }
    }
}
