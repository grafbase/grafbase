use schema::{InputValue, InputValueSerdeError};
use serde::{de::Visitor, forward_to_deserialize_any};

use crate::operation::VariableDefinitionId;

use super::OperationWalker;

pub type VariableWalker<'a> = OperationWalker<'a, VariableDefinitionId, ()>;

impl<'a> VariableWalker<'a> {
    pub fn is_undefined(&self) -> bool {
        self.variables[self.item].is_none() && self.as_ref().default_value.is_none()
    }

    pub fn to_input_value(self) -> Option<InputValue<'a>> {
        self.variables[self.item]
            .map(|id| self.walk(&self.variables[id]).into())
            .or_else(|| {
                self.as_ref()
                    .default_value
                    .map(|id| self.walk(&self.operation.query_input_values[id]).into())
            })
    }
}

impl<'a> serde::Serialize for VariableWalker<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if let Some(id) = self.variables[self.item] {
            self.walk(&self.variables[id]).serialize(serializer)
        } else if let Some(id) = self.as_ref().default_value {
            self.walk(&self.operation.query_input_values[id]).serialize(serializer)
        } else {
            serializer.serialize_none()
        }
    }
}

impl<'de> serde::Deserializer<'de> for VariableWalker<'de> {
    type Error = InputValueSerdeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(id) = self.variables[self.item] {
            self.walk(&self.variables[id]).deserialize_any(visitor)
        } else if let Some(id) = self.as_ref().default_value {
            self.walk(&self.operation.query_input_values[id])
                .deserialize_any(visitor)
        } else {
            visitor.visit_none()
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(id) = self.variables[self.item] {
            self.walk(&self.variables[id]).deserialize_option(visitor)
        } else if let Some(id) = self.as_ref().default_value {
            self.walk(&self.operation.query_input_values[id])
                .deserialize_option(visitor)
        } else {
            visitor.visit_none()
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

impl std::fmt::Debug for VariableWalker<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(id) = self.variables[self.item] {
            f.debug_tuple("Variable")
                .field(&self.walk(&self.variables[id]))
                .finish()
        } else if let Some(id) = self.as_ref().default_value {
            f.debug_tuple("Query").field(&self.walk(&self.operation[id])).finish()
        } else {
            f.debug_struct("Undefined").finish()
        }
    }
}
