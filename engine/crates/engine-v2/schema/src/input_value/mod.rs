use crate::{InputValueDefinitionId, SchemaWalker, StringId};

mod display;
mod ser;
pub use display::DisplayableInpuValue;
pub use ser::SerializableInputValue;

#[derive(Debug, Clone)]
pub enum InputValue {
    Null,
    String(Box<str>),
    StringId(StringId),
    Int(i32),
    BigInt(i64),
    Float(f64),
    Boolean(bool),
    // Ordering is undefined for now, if we need PartialEq we should probably consider sorting it.
    Object(Box<[(InputValueDefinitionId, InputValue)]>),
    List(Box<[InputValue]>),
    Json(Box<serde_json::Value>),
}

// We could have a InputValueWalker for Display, but for serde we might want later to allow
// resolvers to customize how a value is serialized (datetime typically).
// And InputValues will also be generated outside the schema (variables), so not entirely sure
// about a Walker for now
impl InputValue {
    pub fn as_serializable<'s, 'w>(&'s self, schema: SchemaWalker<'w, ()>) -> SerializableInputValue<'s>
    where
        'w: 's,
    {
        SerializableInputValue { schema, value: self }
    }

    pub fn as_displayable<'s, 'w>(&'s self, schema: SchemaWalker<'w, ()>) -> DisplayableInpuValue<'s>
    where
        'w: 's,
    {
        DisplayableInpuValue { schema, value: self }
    }
}

impl From<&str> for InputValue {
    fn from(s: &str) -> Self {
        Self::String(s.into())
    }
}

impl From<String> for InputValue {
    fn from(s: String) -> Self {
        Self::String(s.into())
    }
}

impl From<StringId> for InputValue {
    fn from(s: StringId) -> Self {
        Self::StringId(s)
    }
}

impl From<i32> for InputValue {
    fn from(i: i32) -> Self {
        Self::Int(i)
    }
}

impl From<i64> for InputValue {
    fn from(i: i64) -> Self {
        Self::BigInt(i)
    }
}

impl From<f64> for InputValue {
    fn from(f: f64) -> Self {
        Self::Float(f)
    }
}

impl From<bool> for InputValue {
    fn from(b: bool) -> Self {
        Self::Boolean(b)
    }
}

impl From<serde_json::Value> for InputValue {
    fn from(v: serde_json::Value) -> Self {
        Self::Json(v.into())
    }
}
