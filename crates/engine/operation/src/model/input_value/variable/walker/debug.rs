use walker::Walk;

use super::{VariableInputValue, VariableInputValueRecord, VariableValue};

impl std::fmt::Debug for VariableValue<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VariableValue::Undefined => f.debug_struct("Undefined").finish(),
            VariableValue::Provided(value) => f.debug_tuple("Provided").field(&value).finish(),
            VariableValue::DefaultValue(value) => f.debug_tuple("DefaultValue").field(&value).finish(),
        }
    }
}

impl std::fmt::Debug for VariableInputValue<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let VariableInputValue { ctx, ref_: value } = *self;
        match value {
            VariableInputValueRecord::Null => write!(f, "Null"),
            VariableInputValueRecord::String(s) => s.fmt(f),
            VariableInputValueRecord::EnumValue(id) => {
                f.debug_tuple("EnumValue").field(&id.walk(ctx.schema).name()).finish()
            }
            VariableInputValueRecord::Int(n) => f.debug_tuple("Int").field(n).finish(),
            VariableInputValueRecord::I64(n) => f.debug_tuple("I64").field(n).finish(),
            VariableInputValueRecord::U64(n) => f.debug_tuple("U64").field(n).finish(),
            VariableInputValueRecord::Float(n) => f.debug_tuple("Float").field(n).finish(),
            VariableInputValueRecord::Boolean(b) => b.fmt(f),
            VariableInputValueRecord::InputObject(ids) => {
                let mut map = f.debug_struct("InputObject");
                for (input_value_definition, value) in ids.walk(ctx) {
                    map.field(input_value_definition.name(), &value);
                }
                map.finish()
            }
            VariableInputValueRecord::List(ids) => f.debug_list().entries(ids.walk(ctx)).finish(),
            VariableInputValueRecord::Map(ids) => f.debug_map().entries(ids.walk(ctx)).finish(),
            VariableInputValueRecord::DefaultValue(id) => id.walk(ctx.schema).fmt(f),
        }
    }
}
