use crate::{EnumValueId, IdRange, InputValueDefinitionId, SchemaWalker, StringId};

mod de;
mod display;
mod error;
mod ids;
mod raw;
mod ser;
mod walker;

pub use error::*;
pub use ids::*;
pub use raw::*;
pub use walker::*;

pub type SchemaInputValues = RawInputValues<StringId>;
pub type SchemaInputValue = RawInputValue<StringId>;
pub type SchemaInputValueId = RawInputValueId<StringId>;
pub type SchemaInputObjectFieldValueId = RawInputObjectFieldValueId<StringId>;
pub type SchemaInputKeyValueId = RawInputKeyValueId<StringId>;
pub type SchemaInputMap = IdRange<RawInputKeyValueId<StringId>>;

/// InputValue to be used during execution if more control is needed that what serde/display can
/// provide for a RawInputValue. With a PlanInputValue `value`, you just need to use
/// `InputValue::from(value)`.
#[derive(Default, Debug, Clone)]
pub enum InputValue<'a> {
    #[default]
    Null,
    String(&'a str),
    EnumValue(EnumValueId),
    Int(i32),
    BigInt(i64),
    Float(f64),
    Boolean(bool),
    // There is no guarantee on the ordering.
    InputObject(Box<[(InputValueDefinitionId, InputValue<'a>)]>),
    List(Box<[InputValue<'a>]>),

    /// for JSON
    Map(Box<[(&'a str, InputValue<'a>)]>), // no guarantee on the ordering
    U64(u64),
}

/// If you need to serialize a whole argument, just serialize the PlanInputValue directly without
/// passing through InputValue. This is only useful if you want to partially serialize an argument.
impl serde::Serialize for SchemaWalker<'_, &InputValue<'_>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match &self.item {
            InputValue::Null => serializer.serialize_none(),
            InputValue::String(s) => s.serialize(serializer),
            InputValue::EnumValue(id) => self.walk(*id).name().serialize(serializer),
            InputValue::Int(n) => n.serialize(serializer),
            InputValue::BigInt(n) => n.serialize(serializer),
            InputValue::Float(f) => f.serialize(serializer),
            InputValue::U64(n) => n.serialize(serializer),
            InputValue::Boolean(b) => b.serialize(serializer),
            InputValue::InputObject(fields) => {
                use serde::ser::SerializeMap;
                let mut map = serializer.serialize_map(Some(fields.len()))?;
                for (key, value) in fields.iter() {
                    map.serialize_entry(&self.walk(*key).name(), &self.walk(value))?;
                }
                map.end()
            }
            InputValue::List(list) => {
                use serde::ser::SerializeSeq;
                let mut seq = serializer.serialize_seq(Some(list.len()))?;
                for value in list.iter() {
                    seq.serialize_element(&self.walk(value))?;
                }
                seq.end()
            }
            InputValue::Map(key_values) => {
                use serde::ser::SerializeMap;
                let mut map = serializer.serialize_map(Some(key_values.len()))?;
                for (key, value) in key_values.iter() {
                    map.serialize_entry(key, &self.walk(value))?;
                }
                map.end()
            }
        }
    }
}
