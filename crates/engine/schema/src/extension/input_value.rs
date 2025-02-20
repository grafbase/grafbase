use crate::{FieldSetRecord, InputValueSet, StringId, TemplateId};

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
    Template(TemplateId),
}
