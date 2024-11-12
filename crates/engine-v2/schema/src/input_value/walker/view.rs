use crate::InputValueSerdeError;

use super::{InputValueSet, SchemaInputValue, SchemaInputValueRecord};
use serde::{
    de::{value::MapDeserializer, IntoDeserializer, Visitor},
    forward_to_deserialize_any,
};
use walker::Walk;

pub struct SchemaInputValueView<'a> {
    pub(super) value: SchemaInputValue<'a>,
    pub(super) selection_set: &'a InputValueSet,
}

impl<'a> serde::Serialize for SchemaInputValueView<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Composition guarantees a proper InputValueSet, so if the selection set is empty it means
        // we're serializing a scalar.
        if self.selection_set.is_empty() {
            return self.value.serialize(serializer);
        }
        let SchemaInputValueRecord::InputObject(fields) = self.value.ref_ else {
            return Err(serde::ser::Error::custom(
                "Can only select fields within an input object.",
            ));
        };
        serializer.collect_map(
            fields
                .walk(self.value.schema)
                .filter_map(|(input_value_definition, value)| {
                    if let Some(item) = self
                        .selection_set
                        .iter()
                        .find(|item| item.id == input_value_definition.id)
                    {
                        let value = Self {
                            value,
                            selection_set: &item.subselection,
                        };
                        Some((input_value_definition.name(), value))
                    } else {
                        None
                    }
                }),
        )
    }
}

impl<'de> serde::Deserializer<'de> for SchemaInputValueView<'de> {
    type Error = InputValueSerdeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        // Composition guarantees a proper InputValueSet, so if the selection set is empty it means
        // we're deserializing a scalar.
        if self.selection_set.is_empty() {
            return self.value.deserialize_any(visitor);
        }

        let SchemaInputValueRecord::InputObject(fields) = self.value.ref_ else {
            return Err(serde::de::Error::custom(
                "Can only select fields within an input object.",
            ));
        };

        MapDeserializer::new(
            fields
                .walk(self.value.schema)
                .filter_map(|(input_value_definition, value)| {
                    if let Some(item) = self
                        .selection_set
                        .iter()
                        .find(|item| item.id == input_value_definition.id)
                    {
                        let value = Self {
                            value,
                            selection_set: &item.subselection,
                        };
                        Some((input_value_definition.name(), value))
                    } else {
                        None
                    }
                }),
        )
        .deserialize_any(visitor)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if matches!(self.value.ref_, SchemaInputValueRecord::Null) {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier
    }
}

impl<'de> IntoDeserializer<'de, InputValueSerdeError> for SchemaInputValueView<'de> {
    type Deserializer = Self;
    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}
