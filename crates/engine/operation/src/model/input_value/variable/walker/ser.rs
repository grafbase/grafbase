use walker::Walk;

use crate::VariableInputValueRecord;

use super::{VariableInputValue, VariableValue};

impl serde::Serialize for VariableValue<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            VariableValue::Undefined => serializer.serialize_none(),
            VariableValue::Provided(walker) => walker.serialize(serializer),
            VariableValue::DefaultValue(walker) => walker.serialize(serializer),
        }
    }
}

impl serde::Serialize for VariableInputValue<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let VariableInputValue { ctx, ref_: value } = *self;
        match value {
            VariableInputValueRecord::Null => serializer.serialize_none(),
            VariableInputValueRecord::String(s) => s.serialize(serializer),
            VariableInputValueRecord::EnumValue(id) => id.walk(ctx.schema).name().serialize(serializer),
            VariableInputValueRecord::Int(n) => n.serialize(serializer),
            VariableInputValueRecord::I64(n) => n.serialize(serializer),
            VariableInputValueRecord::Float(f) => f.serialize(serializer),
            VariableInputValueRecord::U64(n) => n.serialize(serializer),
            VariableInputValueRecord::Boolean(b) => b.serialize(serializer),
            VariableInputValueRecord::InputObject(ids) => serializer.collect_map(
                ids.walk(ctx)
                    .map(|(input_value_definition, value)| (input_value_definition.name(), value)),
            ),
            VariableInputValueRecord::List(ids) => serializer.collect_seq(ids.walk(ctx)),
            VariableInputValueRecord::Map(ids) => serializer.collect_map(ids.walk(ctx)),
            VariableInputValueRecord::DefaultValue(id) => id.walk(ctx.schema).serialize(serializer),
        }
    }
}
