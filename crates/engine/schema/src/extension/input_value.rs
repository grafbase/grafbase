use walker::Walk;

use crate::{FieldSetRecord, InputValueSet, Schema, StringId};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum ExtensionInputValueRecord {
    Null,
    String(StringId),
    // I don't think we need to distinguish between EnumValue and a string, but we'll see.
    EnumValue(StringId),
    Int(i32),
    BigInt(i64),
    U64(u64),
    Float(f64),
    Boolean(bool),
    Map(Vec<(StringId, ExtensionInputValueRecord)>),
    List(Vec<ExtensionInputValueRecord>),

    // For data injection
    FieldSet(FieldSetRecord),
    InputValueSet(InputValueSet),
}

#[derive(Clone, Copy)]
pub struct StaticExtensionInputValue<'a> {
    pub(crate) schema: &'a Schema,
    pub ref_: &'a ExtensionInputValueRecord,
}

impl<'a> Walk<&'a Schema> for &ExtensionInputValueRecord {
    type Walker<'w>
        = StaticExtensionInputValue<'w>
    where
        Self: 'w,
        'a: 'w;
    fn walk<'w>(self, schema: impl Into<&'a Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        StaticExtensionInputValue {
            schema: schema.into(),
            ref_: self,
        }
    }
}

impl std::fmt::Debug for StaticExtensionInputValue<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StaticExtensionInputValue").finish_non_exhaustive()
    }
}

impl serde::Serialize for StaticExtensionInputValue<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self.ref_ {
            ExtensionInputValueRecord::Null => serializer.serialize_none(),
            ExtensionInputValueRecord::String(id) => serializer.serialize_str(&self.schema[*id]),
            ExtensionInputValueRecord::EnumValue(id) => serializer.serialize_str(&self.schema[*id]),
            ExtensionInputValueRecord::Int(value) => serializer.serialize_i32(*value),
            ExtensionInputValueRecord::BigInt(value) => serializer.serialize_i64(*value),
            ExtensionInputValueRecord::U64(value) => serializer.serialize_u64(*value),
            ExtensionInputValueRecord::Float(value) => serializer.serialize_f64(*value),
            ExtensionInputValueRecord::Boolean(value) => serializer.serialize_bool(*value),
            ExtensionInputValueRecord::Map(map) => serializer.collect_map(
                map.iter()
                    .map(|(key, value)| (&self.schema[*key], value.walk(self.schema))),
            ),
            ExtensionInputValueRecord::List(list) => {
                serializer.collect_seq(list.iter().map(|value| value.walk(self.schema)))
            }
            ExtensionInputValueRecord::FieldSet(_) | ExtensionInputValueRecord::InputValueSet(_) => {
                unreachable!("Invariant broken, cannot be a static value.")
            }
        }
    }
}
