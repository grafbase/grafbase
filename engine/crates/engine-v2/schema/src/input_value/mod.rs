use crate::{EnumValueId, IdRange, InputValueDefinitionId, SchemaWalker, StringId};

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

#[derive(Default)]
pub struct SchemaInputValues {
    /// Inidividual input values and list values
    values: Vec<SchemaInputValue>,
    /// InputObject's fields
    input_fields: Vec<(InputValueDefinitionId, SchemaInputValue)>,
    /// Object's fields (for JSON)
    key_values: Vec<(StringId, SchemaInputValue)>,
}

id_newtypes::U32! {
    SchemaInputValues.values[SchemaInputValueId] => SchemaInputValue,
    SchemaInputValues.input_fields[SchemaInputObjectFieldValueId] => (InputValueDefinitionId, SchemaInputValue),
    SchemaInputValues.key_values[SchemaInputKeyValueId] => (StringId, SchemaInputValue),
}

/// Represents a default input value and @requires arguments.
#[derive(Debug, Copy, Clone)]
pub enum SchemaInputValue {
    Null,
    String(StringId),
    EnumValue(EnumValueId),
    Int(i32),
    BigInt(i64),
    Float(f64),
    Boolean(bool),
    InputObject(IdRange<SchemaInputObjectFieldValueId>),
    List(IdRange<SchemaInputValueId>),

    /// for JSON
    Map(IdRange<SchemaInputKeyValueId>),
    U64(u64),
}

impl SchemaInputValues {
    pub fn push_value(&mut self, value: SchemaInputValue) -> SchemaInputValueId {
        let id = SchemaInputValueId::from(self.values.len());
        self.values.push(value);
        id
    }

    #[cfg(test)]
    pub fn push_list(&mut self, values: Vec<SchemaInputValue>) -> IdRange<SchemaInputValueId> {
        let start = self.values.len();
        self.values.extend(values);
        (start..self.values.len()).into()
    }

    /// Reserve InputValue slots for a list, avoiding the need for an intermediate
    /// Vec to hold values as we need them to be contiguous.
    pub fn reserve_list(&mut self, n: usize) -> IdRange<SchemaInputValueId> {
        let start = self.values.len();
        self.values.reserve(n);
        for _ in 0..n {
            self.values.push(SchemaInputValue::Null);
        }
        (start..self.values.len()).into()
    }

    #[cfg(test)]
    pub fn push_map(&mut self, fields: Vec<(StringId, SchemaInputValue)>) -> IdRange<SchemaInputKeyValueId> {
        let start = self.key_values.len();
        self.key_values.extend(fields);
        (start..self.key_values.len()).into()
    }

    /// Reserve InputKeyValue slots for a map, avoiding the need for an intermediate
    /// Vec to hold values as we need them to be contiguous.
    pub fn reserve_map(&mut self, n: usize) -> IdRange<SchemaInputKeyValueId> {
        let start = self.key_values.len();
        self.key_values.reserve(n);
        for _ in 0..n {
            self.key_values.push((StringId::from(0), SchemaInputValue::Null));
        }
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

    /// Reserve InputObjectFieldValue slots for an InputObject, avoiding the need for an intermediate
    /// Vec to hold values as we need them to be contiguous.
    pub fn reserve_input_object(&mut self, n: usize) -> IdRange<SchemaInputObjectFieldValueId> {
        let start = self.input_fields.len();
        self.input_fields.reserve(n);
        for _ in 0..n {
            self.input_fields
                .push((InputValueDefinitionId::from(0), SchemaInputValue::Null));
        }
        (start..self.input_fields.len()).into()
    }
}
