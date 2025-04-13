use walker::Walk;

use crate::{InputValueContext, QueryInputValueRecord, VariableDefinitionId};

use super::{QueryInputValue, QueryOrSchemaInputValue};

impl serde::Serialize for QueryInputValue<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let QueryInputValue { ctx, ref_: value } = *self;
        match value {
            QueryInputValueRecord::Null => serializer.serialize_none(),
            QueryInputValueRecord::String(s) | QueryInputValueRecord::UnboundEnumValue(s) => s.serialize(serializer),
            QueryInputValueRecord::EnumValue(id) => id.walk(ctx.schema).name().serialize(serializer),
            QueryInputValueRecord::Int(n) => n.serialize(serializer),
            QueryInputValueRecord::I64(n) => n.serialize(serializer),
            QueryInputValueRecord::Float(f) => f.serialize(serializer),
            QueryInputValueRecord::U64(n) => n.serialize(serializer),
            QueryInputValueRecord::Boolean(b) => b.serialize(serializer),
            QueryInputValueRecord::InputObject(ids) => {
                serializer.collect_map(ids.walk(ctx).filter_map(|(input_value_definition, value)| {
                    if value.is_undefined() {
                        input_value_definition
                            .default_value()
                            .map(|value| (input_value_definition.name(), QueryOrSchemaInputValue::Schema(value)))
                    } else {
                        Some((input_value_definition.name(), QueryOrSchemaInputValue::Query(value)))
                    }
                }))
            }
            QueryInputValueRecord::List(ids) => serializer.collect_seq(ids.walk(ctx)),
            QueryInputValueRecord::Map(ids) => serializer.collect_map(ids.walk(ctx)),
            QueryInputValueRecord::DefaultValue(id) => id.walk(ctx.schema).serialize(serializer),
            QueryInputValueRecord::Variable(id) => {
                <VariableDefinitionId as Walk<InputValueContext<'_>>>::walk(*id, self.ctx).serialize(serializer)
            }
        }
    }
}

impl serde::Serialize for QueryOrSchemaInputValue<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            QueryOrSchemaInputValue::Query(value) => value.serialize(serializer),
            QueryOrSchemaInputValue::Schema(value) => value.serialize(serializer),
        }
    }
}
