use federated_graph::Value;

use crate::{SchemaInputValue, SchemaInputValues};

use super::BuildContext;

impl SchemaInputValues {
    pub(crate) fn ingest_arbitrary_federated_value(&mut self, ctx: &BuildContext, value: Value) -> SchemaInputValue {
        match value {
            Value::Null => SchemaInputValue::Null,
            Value::String(id) | Value::UnboundEnumValue(id) => SchemaInputValue::String(id.into()),
            Value::Int(n) => SchemaInputValue::BigInt(n),
            Value::Float(f) => SchemaInputValue::Float(f),
            Value::Boolean(b) => SchemaInputValue::Boolean(b),
            Value::EnumValue(id) => {
                let id = ctx.idmaps.enum_values.get(id).expect("enum ids to be valid");
                SchemaInputValue::EnumValue(id)
            }
            Value::Object(fields) => {
                let ids = self.reserve_map(fields.len());
                for ((name, value), id) in fields.into_vec().into_iter().zip(ids) {
                    self[id] = (name.into(), self.ingest_arbitrary_federated_value(ctx, value));
                }
                self[ids].sort_unstable_by(|(left_key, _), (right_key, _)| {
                    ctx.strings.get_by_id(*left_key).cmp(&ctx.strings.get_by_id(*right_key))
                });
                SchemaInputValue::Map(ids)
            }
            Value::List(list) => {
                let ids = self.reserve_list(list.len());
                for (value, id) in list.into_vec().into_iter().zip(ids) {
                    let value = self.ingest_arbitrary_federated_value(ctx, value);
                    self[id] = value;
                }
                SchemaInputValue::List(ids)
            }
        }
    }
}
