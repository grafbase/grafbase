use crate::{
    prepare::RequiredFieldSetItem,
    response::{ResponseObject, ResponseValue},
};
use schema::{EntityDefinition, InputValueSerdeError};
use serde::{
    de::{
        IntoDeserializer, MapAccess, Visitor,
        value::{MapDeserializer, SeqDeserializer},
    },
    forward_to_deserialize_any,
};
use std::iter::Iterator;

use super::{ResponseObjectView, ResponseValueView, ViewContext};

impl<'de> serde::Deserializer<'de> for ResponseObjectView<'de> {
    type Error = InputValueSerdeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let map = ResponseObjectViewMapAcces {
            ctx: self.ctx,
            response_object: self.response_object,
            selection: None,
            selection_set: self.selection_set.iter(),
        };
        visitor.visit_map(map)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_some(self)
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

struct ResponseObjectViewMapAcces<'de, I> {
    ctx: ViewContext<'de>,
    response_object: &'de ResponseObject,
    selection: Option<(&'de ResponseValue, RequiredFieldSetItem<'de>)>,
    selection_set: I,
}

impl<'de, I> MapAccess<'de> for ResponseObjectViewMapAcces<'de, I>
where
    I: Iterator<Item = RequiredFieldSetItem<'de>>,
{
    type Error = InputValueSerdeError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        for selection in self.selection_set.by_ref() {
            let field = selection.data_field();
            let key = field.definition().name();
            let value = match self.response_object.find_by_response_key(field.response_key) {
                Some(value) => value,
                None => {
                    // If this field doesn't match the actual response object, meaning this field
                    // was in a fragment that doesn't apply to this object, we can safely skip it.
                    if let Some(definition_id) = self.response_object.definition_id {
                        match field.definition().parent_entity() {
                            EntityDefinition::Interface(inf) => {
                                if inf.possible_type_ids.binary_search(&definition_id).is_err() {
                                    continue;
                                }
                            }
                            EntityDefinition::Object(obj) => {
                                if obj.id != definition_id {
                                    continue;
                                }
                            }
                        }
                    }

                    return Err(InputValueSerdeError::Message(format!(
                        "Could not retrieve field {}.{key}",
                        field.definition().parent_entity().name(),
                    )));
                }
            };
            self.selection = Some((value, selection));
            return seed.deserialize(key.into_deserializer()).map(Some);
        }

        Ok(None)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        // Panic because this indicates a bug in the program rather than an
        // expected failure.
        let (value, selection) = self
            .selection
            .take()
            .expect("MapAccess::next_value called before next_key");
        let value = ResponseValueView {
            ctx: self.ctx,
            value,
            selection_set: selection.subselection(),
        };
        seed.deserialize(value.into_deserializer())
    }
}

impl<'de> IntoDeserializer<'de, InputValueSerdeError> for ResponseObjectView<'de> {
    type Deserializer = Self;
    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

impl<'de> serde::Deserializer<'de> for ResponseValueView<'de> {
    type Error = InputValueSerdeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            ResponseValue::Null => visitor.visit_none(),
            ResponseValue::Inaccessible { id } => ResponseValueView {
                ctx: self.ctx,
                value: &self.ctx.response.data_parts[*id],
                selection_set: self.selection_set,
            }
            .deserialize_any(visitor),
            ResponseValue::Boolean { value, .. } => visitor.visit_bool(*value),
            ResponseValue::Int { value, .. } => visitor.visit_i32(*value),
            ResponseValue::Float { value, .. } => visitor.visit_f64(*value),
            ResponseValue::String { value, .. } => visitor.visit_borrowed_str(value),
            ResponseValue::StringId { id, .. } => visitor.visit_borrowed_str(&self.ctx.response.schema[*id]),
            &ResponseValue::List { id, .. } => {
                let values = &self.ctx.response.data_parts[id];

                SeqDeserializer::new(values.iter().map(|value| ResponseValueView {
                    ctx: self.ctx,
                    value,
                    selection_set: self.selection_set,
                }))
                .deserialize_any(visitor)
            }
            &ResponseValue::Object { id, .. } => ResponseObjectView {
                ctx: self.ctx,
                response_object: &self.ctx.response.data_parts[id],
                selection_set: self.selection_set,
            }
            .deserialize_any(visitor),
            ResponseValue::Unexpected => Err(InputValueSerdeError::Message("Unexpected value".to_string())),
            ResponseValue::I64 { value } => visitor.visit_i64(*value),
            ResponseValue::U64 { value } => visitor.visit_u64(*value),
            ResponseValue::Map { id } => {
                MapDeserializer::new(self.ctx.response.data_parts[*id].iter().map(|(key, value)| {
                    (
                        key.as_str(),
                        ResponseValueView {
                            ctx: self.ctx,
                            value,
                            selection_set: self.selection_set,
                        },
                    )
                }))
                .deserialize_any(visitor)
            }
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_some(self)
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

impl<'de> IntoDeserializer<'de, InputValueSerdeError> for ResponseValueView<'de> {
    type Deserializer = Self;
    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}
