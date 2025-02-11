use federated_graph::Value;

use crate::SchemaInputValueRecord;

use super::GraphContext;

impl GraphContext<'_> {
    pub(crate) fn ingest_arbitrary_value(&mut self, value: Value) -> SchemaInputValueRecord {
        match value {
            Value::Null => SchemaInputValueRecord::Null,
            Value::String(id) => SchemaInputValueRecord::String(self.get_or_insert_str(id)),
            Value::UnboundEnumValue(id) => SchemaInputValueRecord::UnboundEnumValue(self.get_or_insert_str(id)),
            Value::Int(n) => SchemaInputValueRecord::BigInt(n),
            Value::Float(f) => SchemaInputValueRecord::Float(f),
            Value::Boolean(b) => SchemaInputValueRecord::Boolean(b),
            Value::EnumValue(id) => {
                let value = self.federated_graph[id].value;
                SchemaInputValueRecord::UnboundEnumValue(self.get_or_insert_str(value))
            }
            Value::Object(fields) => {
                let ids = self.graph.input_values.reserve_map(fields.len());
                for ((name, value), id) in fields.into_vec().into_iter().zip(ids) {
                    let name = self.get_or_insert_str(name);
                    self.graph.input_values[id] = (name, self.ingest_arbitrary_value(value));
                }
                let ctx = &self.ctx;
                self.graph.input_values[ids].sort_unstable_by(|(left_key, _), (right_key, _)| {
                    ctx.strings.get_by_id(*left_key).cmp(&ctx.strings.get_by_id(*right_key))
                });
                SchemaInputValueRecord::Map(ids)
            }
            Value::List(list) => {
                let ids = self.graph.input_values.reserve_list(list.len());
                for (value, id) in list.into_vec().into_iter().zip(ids) {
                    let value = self.ingest_arbitrary_value(value);
                    self.graph.input_values[id] = value;
                }
                SchemaInputValueRecord::List(ids)
            }
        }
    }
}
