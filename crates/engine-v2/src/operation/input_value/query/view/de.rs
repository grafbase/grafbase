use schema::InputValueSerdeError;
use serde::{
    de::{value::MapDeserializer, IntoDeserializer, Visitor},
    forward_to_deserialize_any,
};
use walker::Walk;

use crate::operation::QueryInputValueRecord;

use super::{QueryInputValueView, QueryOrSchemaInputValueView};

impl<'de> serde::Deserializer<'de> for QueryInputValueView<'de> {
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

        let QueryInputValueRecord::InputObject(fields) = self.value.ref_ else {
            return Err(serde::de::Error::custom(
                "Can only select fields within an input object.",
            ));
        };

        MapDeserializer::new(
            fields
                .walk(self.value.ctx)
                .filter_map(|(input_value_definition, value)| {
                    if let Some(item) = self
                        .selection_set
                        .iter()
                        .find(|item| item.id == input_value_definition.id)
                    {
                        if value.is_undefined() {
                            input_value_definition.default_value().map(|value| {
                                (
                                    input_value_definition.name(),
                                    QueryOrSchemaInputValueView::Schema(value.with_selection_set(&item.subselection)),
                                )
                            })
                        } else {
                            Some((
                                input_value_definition.name(),
                                QueryOrSchemaInputValueView::Query(value.with_selection_set(&item.subselection)),
                            ))
                        }
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
        if matches!(self.value.ref_, QueryInputValueRecord::Null) {
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

impl<'de> IntoDeserializer<'de, InputValueSerdeError> for QueryInputValueView<'de> {
    type Deserializer = Self;
    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

impl<'de> serde::Deserializer<'de> for QueryOrSchemaInputValueView<'de> {
    type Error = InputValueSerdeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            QueryOrSchemaInputValueView::Query(value) => value.deserialize_any(visitor),
            QueryOrSchemaInputValueView::Schema(value) => value.deserialize_any(visitor),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            QueryOrSchemaInputValueView::Query(value) => value.deserialize_option(visitor),
            QueryOrSchemaInputValueView::Schema(value) => value.deserialize_option(visitor),
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

impl<'de> IntoDeserializer<'de, InputValueSerdeError> for QueryOrSchemaInputValueView<'de> {
    type Deserializer = Self;
    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}
