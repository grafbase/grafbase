use schema::{InputValueSerdeError, InputValueSet};
use serde::{
    de::{value::MapDeserializer, IntoDeserializer},
    forward_to_deserialize_any,
    ser::SerializeMap,
};

use super::FieldArgumentsWalker;

pub struct FieldArgumentsView<'a> {
    pub(super) inner: FieldArgumentsWalker<'a>,
    pub(super) selection_set: &'a InputValueSet,
}

impl<'a> serde::Serialize for FieldArgumentsView<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        for item in self.selection_set.iter() {
            for id in self.inner.item {
                if self.inner.operation[id].input_value_definition_id == item.id {
                    let arg = self.inner.walk(id);
                    if let Some(value) = arg.value() {
                        map.serialize_key(arg.definition().name())?;
                        map.serialize_value(&value.with_selection_set(&item.subselection))?;
                    }
                    continue;
                }
            }
        }
        map.end()
    }
}

impl<'de> serde::Deserializer<'de> for FieldArgumentsView<'de> {
    type Error = InputValueSerdeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        MapDeserializer::new(self.selection_set.iter().filter_map(|item| {
            self.inner.item.into_iter().find_map(|id| {
                if self.inner.operation[id].input_value_definition_id == item.id {
                    let arg = self.inner.walk(id);
                    let value = arg.value()?;
                    Some((arg.definition().name(), value.with_selection_set(&item.subselection)))
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

impl<'de> IntoDeserializer<'de, InputValueSerdeError> for FieldArgumentsView<'de> {
    type Deserializer = Self;
    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}
