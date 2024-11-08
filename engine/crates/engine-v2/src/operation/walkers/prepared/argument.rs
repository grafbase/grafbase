use id_newtypes::{IdRange, IdRangeIterator};
use schema::{InputValueDefinition, InputValueSerdeError, InputValueSet};
use serde::{de::value::MapDeserializer, forward_to_deserialize_any};

use crate::operation::{BoundFieldArgumentId, QueryInputValueWalker};

mod view;

pub(crate) use view::*;

use super::PreparedOperationWalker;

pub type FieldArgumentWalker<'a> = PreparedOperationWalker<'a, BoundFieldArgumentId>;

impl<'a> FieldArgumentWalker<'a> {
    pub fn value(&self) -> Option<QueryInputValueWalker<'a>> {
        let value = self.walk(&self.operation.query_input_values[self.as_ref().input_value_id]);
        if value.is_undefined() {
            None
        } else {
            Some(value)
        }
    }

    pub fn definition(&self) -> InputValueDefinition<'a> {
        self.schema.walk(self.operation[self.item].input_value_definition_id)
    }
}

impl std::fmt::Debug for FieldArgumentWalker<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FieldArgumentWalker")
            .field("name", &self.definition().name())
            .field("value", &self.value())
            .finish()
    }
}

pub type FieldArgumentsWalker<'a> = PreparedOperationWalker<'a, IdRange<BoundFieldArgumentId>>;

impl<'a> FieldArgumentsWalker<'a> {
    pub fn is_empty(&self) -> bool {
        self.item.is_empty()
    }

    pub fn with_selection_set(self, selection_set: &'a InputValueSet) -> FieldArgumentsView<'a> {
        FieldArgumentsView {
            inner: self,
            selection_set,
        }
    }
}

impl<'a> IntoIterator for FieldArgumentsWalker<'a> {
    type Item = FieldArgumentWalker<'a>;

    type IntoIter = FieldArgumentsIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        FieldArgumentsIterator(self.walk(self.item.into_iter()))
    }
}

pub(crate) struct FieldArgumentsIterator<'a>(PreparedOperationWalker<'a, IdRangeIterator<BoundFieldArgumentId>>);

impl<'a> Iterator for FieldArgumentsIterator<'a> {
    type Item = FieldArgumentWalker<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.item.next().map(|id| self.0.walk(id))
    }
}

impl ExactSizeIterator for FieldArgumentsIterator<'_> {
    fn len(&self) -> usize {
        self.0.item.len()
    }
}

impl<'de> serde::Deserializer<'de> for FieldArgumentsWalker<'de> {
    type Error = InputValueSerdeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        MapDeserializer::new(self.into_iter().filter_map(|arg| {
            let value = arg.value()?;
            Some((arg.definition().name(), value))
        }))
        .deserialize_any(visitor)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier option ignored_any
    }
}
