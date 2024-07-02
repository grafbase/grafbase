use crate::{EnumValueId, IdRange, InputValueDefinitionId, Schema, SchemaWalker, StringId};

mod de;
mod display;
mod error;
mod ser;
#[cfg(test)]
mod tests;
mod walker;

pub use error::*;
pub use walker::*;

/// implement a Deserializer & Serialize trait, but if you need to traverse a dynamic type,
/// this will be the one to use. All input values can be converted to it.
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

/// Provided if you need to serialize only a part of an input value.
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

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct SchemaInputValues {
    /// Individual input values and list values
    values: Vec<SchemaInputValue>,
    /// InputObject's fields
    input_fields: Vec<(InputValueDefinitionId, SchemaInputValue)>,
    /// Object's fields (for JSON)
    key_values: Vec<(StringId, SchemaInputValue)>,
}

id_newtypes::NonZeroU32! {
    SchemaInputValues.values[SchemaInputValueId] => SchemaInputValue | index(Schema.graph.input_values),
    SchemaInputValues.input_fields[SchemaInputObjectFieldValueId] => (InputValueDefinitionId, SchemaInputValue) | index(Schema.graph.input_values),
    SchemaInputValues.key_values[SchemaInputKeyValueId] => (StringId, SchemaInputValue) | index(Schema.graph.input_values),
}

/// Represents a default input value and @requires arguments.
#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub enum SchemaInputValue {
    Null,
    String(StringId),
    EnumValue(EnumValueId),
    Int(i32),
    BigInt(i64),
    Float(f64),
    Boolean(bool),
    // sorted by input_value_definition_id
    InputObject(IdRange<SchemaInputObjectFieldValueId>),
    List(IdRange<SchemaInputValueId>),

    // for JSON
    // sorted by the key (actual String, not the StringId)
    Map(IdRange<SchemaInputKeyValueId>),
    U64(u64),
}

impl SchemaInputValue {
    fn discriminant(&self) -> u8 {
        match self {
            SchemaInputValue::Null => 0,
            SchemaInputValue::String(_) => 1,
            SchemaInputValue::EnumValue(_) => 2,
            SchemaInputValue::Int(_) => 3,
            SchemaInputValue::BigInt(_) => 4,
            SchemaInputValue::Float(_) => 5,
            SchemaInputValue::Boolean(_) => 6,
            SchemaInputValue::InputObject(_) => 7,
            SchemaInputValue::List(_) => 8,
            SchemaInputValue::Map(_) => 9,
            SchemaInputValue::U64(_) => 10,
        }
    }
}

impl SchemaInputValues {
    pub(crate) fn push_value(&mut self, value: SchemaInputValue) -> SchemaInputValueId {
        let id = SchemaInputValueId::from(self.values.len());
        self.values.push(value);
        id
    }

    /// Reserve InputValue slots for a list, avoiding the need for an intermediate
    /// Vec to hold values as we need them to be contiguous.
    pub(crate) fn reserve_list(&mut self, n: usize) -> IdRange<SchemaInputValueId> {
        let start = self.values.len();
        self.values.reserve(n);
        for _ in 0..n {
            self.values.push(SchemaInputValue::Null);
        }
        (start..self.values.len()).into()
    }

    /// Reserve InputKeyValue slots for a map, avoiding the need for an intermediate
    /// Vec to hold values as we need them to be contiguous.
    pub(crate) fn reserve_map(&mut self, n: usize) -> IdRange<SchemaInputKeyValueId> {
        let start = self.key_values.len();
        self.key_values.reserve(n);
        for _ in 0..n {
            self.key_values.push((StringId::from(0), SchemaInputValue::Null));
        }
        (start..self.key_values.len()).into()
    }

    pub(crate) fn append_input_object(
        &mut self,
        fields: &mut Vec<(InputValueDefinitionId, SchemaInputValue)>,
    ) -> IdRange<SchemaInputObjectFieldValueId> {
        let start = self.input_fields.len();
        self.input_fields.append(fields);
        (start..self.input_fields.len()).into()
    }
}

#[cfg(test)]
impl SchemaInputValues {
    pub fn push_list(&mut self, values: Vec<SchemaInputValue>) -> IdRange<SchemaInputValueId> {
        let start = self.values.len();
        self.values.extend(values);
        (start..self.values.len()).into()
    }

    pub fn push_map(&mut self, fields: Vec<(StringId, SchemaInputValue)>) -> IdRange<SchemaInputKeyValueId> {
        let start = self.key_values.len();
        self.key_values.extend(fields);
        (start..self.key_values.len()).into()
    }

    pub fn push_input_object(
        &mut self,
        fields: impl IntoIterator<Item = (InputValueDefinitionId, SchemaInputValue)>,
    ) -> IdRange<SchemaInputObjectFieldValueId> {
        let start = self.input_fields.len();
        self.input_fields.extend(fields);
        (start..self.input_fields.len()).into()
    }
}
