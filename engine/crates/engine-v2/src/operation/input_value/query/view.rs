use schema::{InputValueSerdeError, InputValueSet};
use serde::{
    de::{value::MapDeserializer, IntoDeserializer},
    forward_to_deserialize_any,
    ser::SerializeMap,
};

use super::{QueryInputValue, QueryInputValueWalker};

pub(crate) struct QueryInputValueView<'a> {
    pub(super) inner: QueryInputValueWalker<'a>,
    pub(super) selection_set: &'a InputValueSet,
}

impl<'a> serde::Serialize for QueryInputValueView<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Composition guarantees a proper InputValueSet, so if the selection set is empty it means
        // we're serializing a scalar.
        if self.selection_set.is_empty() {
            return self.inner.serialize(serializer);
        }
        let QueryInputValue::InputObject(fields) = self.inner.item else {
            return Err(serde::ser::Error::custom(
                "Can only select fields within an input object.",
            ));
        };
        let mut map = serializer.serialize_map(None)?;
        for item in self.selection_set.iter() {
            for (id, value) in &self.inner.operation[*fields] {
                if *id == item.id {
                    map.serialize_key(self.inner.schema_walker.walk(*id).name())?;
                    map.serialize_value(&Self {
                        inner: self.inner.walk(value),
                        selection_set: &item.subselection,
                    })?;
                    continue;
                }
            }
        }
        map.end()
    }
}

impl<'de> serde::Deserializer<'de> for QueryInputValueView<'de> {
    type Error = InputValueSerdeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        // Composition guarantees a proper InputValueSet, so if the selection set is empty it means
        // we're deserializing a scalar.
        if self.selection_set.is_empty() {
            return self.inner.deserialize_any(visitor);
        }

        let QueryInputValue::InputObject(fields) = self.inner.item else {
            return Err(serde::de::Error::custom(
                "Can only select fields within an input object.",
            ));
        };

        MapDeserializer::new(self.selection_set.iter().filter_map(|item| {
            self.inner.operation[*fields].iter().find_map(|(id, value)| {
                if *id == item.id {
                    let name = self.inner.schema_walker.walk(*id).name();
                    let value = Self {
                        inner: self.inner.walk(value),
                        selection_set: &item.subselection,
                    };
                    Some((name, value))
                } else {
                    None
                }
            })
        }))
        .deserialize_any(visitor)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier option ignored_any
    }
}

impl<'de> IntoDeserializer<'de, InputValueSerdeError> for QueryInputValueView<'de> {
    type Deserializer = Self;
    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}
