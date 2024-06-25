use schema::{InputValue, InputValueSerdeError};
use serde::{de::Visitor, forward_to_deserialize_any};

use crate::operation::{QueryInputValueWalker, VariableDefinitionId, VariableInputValueWalker, VariableValue};

use super::OperationWalker;

pub type VariableWalker<'a> = OperationWalker<'a, VariableDefinitionId, ()>;

impl<'a> VariableWalker<'a> {
    // FIXME: Unnecessary indirection...
    pub fn as_value(&self) -> VariableValueWalker<'a> {
        match self.variables[self.item] {
            VariableValue::Unavailable => VariableValueWalker::Unavailable,
            VariableValue::Undefined => {
                if let Some(id) = self.as_ref().default_value {
                    VariableValueWalker::DefaultValue(self.walk(&self.operation.query_input_values[id]))
                } else {
                    VariableValueWalker::Undefined
                }
            }
            VariableValue::InputValue(id) => VariableValueWalker::VariableInputValue(self.walk(&self.variables[id])),
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum VariableValueWalker<'a> {
    Unavailable,
    Undefined,
    VariableInputValue(VariableInputValueWalker<'a>),
    DefaultValue(QueryInputValueWalker<'a>),
}

impl<'a> VariableValueWalker<'a> {
    pub fn is_undefined(&self) -> bool {
        matches!(self, Self::Undefined)
    }

    pub fn to_input_value(self) -> Option<InputValue<'a>> {
        match self {
            Self::VariableInputValue(walker) => Some(walker.into()),
            Self::DefaultValue(walker) => Some(walker.into()),
            Self::Undefined => None,
            Self::Unavailable => unreachable!("Variable value cannot be accessed at this stage."),
        }
    }
}

impl<'a> serde::Serialize for VariableWalker<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.as_value() {
            VariableValueWalker::Unavailable => unreachable!("Variable value cannot be accessed at this stage."),
            VariableValueWalker::Undefined => serializer.serialize_none(),
            VariableValueWalker::VariableInputValue(walker) => walker.serialize(serializer),
            VariableValueWalker::DefaultValue(walker) => walker.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserializer<'de> for VariableWalker<'de> {
    type Error = InputValueSerdeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.as_value() {
            VariableValueWalker::Unavailable => unreachable!("Variable value cannot be accessed at this stage."),
            VariableValueWalker::Undefined => visitor.visit_none(),
            VariableValueWalker::VariableInputValue(walker) => walker.deserialize_any(visitor),
            VariableValueWalker::DefaultValue(walker) => walker.deserialize_any(visitor),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.as_value() {
            VariableValueWalker::Unavailable => unreachable!("Variable value cannot be accessed at this stage."),
            VariableValueWalker::Undefined => visitor.visit_none(),
            VariableValueWalker::VariableInputValue(walker) => walker.deserialize_option(visitor),
            VariableValueWalker::DefaultValue(walker) => walker.deserialize_option(visitor),
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
        match self.as_value() {
            VariableValueWalker::Unavailable => f.debug_struct("Unavailable").finish(),
            VariableValueWalker::Undefined => f.debug_struct("Undefined").finish(),
            VariableValueWalker::VariableInputValue(walker) => {
                f.debug_tuple("VariableInputValue").field(&walker).finish()
            }
            VariableValueWalker::DefaultValue(walker) => f.debug_tuple("DefaultValue").field(&walker).finish(),
        }
    }
}
