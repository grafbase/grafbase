use federated_graph::Value;

use crate::{SchemaInputValueRecord, SchemaInputValues};

use super::BuildContext;

impl SchemaInputValues {
    pub(crate) fn ingest_arbitrary_value(&mut self, ctx: &BuildContext<'_>, value: Value) -> SchemaInputValueRecord {
        match value {
            Value::Null => SchemaInputValueRecord::Null,
            Value::String(id) => SchemaInputValueRecord::String(id.into()),
            Value::UnboundEnumValue(id) => SchemaInputValueRecord::UnboundEnumValue(id.into()),
            Value::Int(n) => SchemaInputValueRecord::BigInt(n),
            Value::Float(f) => SchemaInputValueRecord::Float(f),
            Value::Boolean(b) => SchemaInputValueRecord::Boolean(b),
            Value::EnumValue(id) => SchemaInputValueRecord::EnumValue(id.into()),
            Value::Object(fields) => {
                let ids = self.reserve_map(fields.len());
                for ((name, value), id) in fields.into_vec().into_iter().zip(ids) {
                    self[id] = (name.into(), self.ingest_arbitrary_value(ctx, value));
                }
                self[ids].sort_unstable_by(|(left_key, _), (right_key, _)| {
                    ctx.strings.get_by_id(*left_key).cmp(&ctx.strings.get_by_id(*right_key))
                });
                SchemaInputValueRecord::Map(ids)
            }
            Value::List(list) => {
                let ids = self.reserve_list(list.len());
                for (value, id) in list.into_vec().into_iter().zip(ids) {
                    let value = self.ingest_arbitrary_value(ctx, value);
                    self[id] = value;
                }
                SchemaInputValueRecord::List(ids)
            }
        }
    }
}
