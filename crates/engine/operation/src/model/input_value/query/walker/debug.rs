use walker::Walk;

use crate::{InputValueContext, VariableDefinitionId};

use super::{QueryInputValue, QueryInputValueRecord};

impl std::fmt::Debug for QueryInputValue<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let QueryInputValue { ctx, ref_: value } = *self;
        match value {
            QueryInputValueRecord::Null => write!(f, "Null"),
            QueryInputValueRecord::String(s) => s.fmt(f),
            QueryInputValueRecord::EnumValue(id) => {
                f.debug_tuple("EnumValue").field(&id.walk(ctx.schema).name()).finish()
            }
            QueryInputValueRecord::UnboundEnumValue(s) => f.debug_tuple("UnboundEnumValue").field(&s).finish(),
            QueryInputValueRecord::Int(n) => f.debug_tuple("Int").field(n).finish(),
            QueryInputValueRecord::BigInt(n) => f.debug_tuple("BigInt").field(n).finish(),
            QueryInputValueRecord::U64(n) => f.debug_tuple("U64").field(n).finish(),
            QueryInputValueRecord::Float(n) => f.debug_tuple("Float").field(n).finish(),
            QueryInputValueRecord::Boolean(b) => b.fmt(f),
            QueryInputValueRecord::InputObject(ids) => {
                let mut map = f.debug_struct("InputObject");
                for (input_value_definition, value) in ids.walk(ctx) {
                    map.field(input_value_definition.name(), &value);
                }
                map.finish()
            }
            QueryInputValueRecord::List(ids) => f.debug_list().entries(ids.walk(ctx)).finish(),
            QueryInputValueRecord::Map(ids) => f.debug_map().entries(ids.walk(ctx)).finish(),
            QueryInputValueRecord::DefaultValue(id) => id.walk(ctx.schema).fmt(f),
            QueryInputValueRecord::Variable(id) => {
                <VariableDefinitionId as Walk<InputValueContext<'_>>>::walk(*id, ctx).fmt(f)
            }
        }
    }
}
