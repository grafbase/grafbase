use federated_graph::Value;

use crate::{SchemaInputValueRecord, SchemaInputValues};

use super::BuildContext;

#[derive(Debug, Clone, Copy)]
pub(crate) struct InaccessibleEnumValue;

impl SchemaInputValues {
    pub(crate) fn ingest_arbitrary_federated_value(
        &mut self,
        ctx: &BuildContext,
        value: Value,
    ) -> Result<SchemaInputValueRecord, InaccessibleEnumValue> {
        Ok(match value {
            Value::Null => SchemaInputValueRecord::Null,
            Value::String(id) | Value::UnboundEnumValue(id) => SchemaInputValueRecord::String(id.into()),
            Value::Int(n) => SchemaInputValueRecord::BigInt(n),
            Value::Float(f) => SchemaInputValueRecord::Float(f),
            Value::Boolean(b) => SchemaInputValueRecord::Boolean(b),
            Value::EnumValue(id) => {
                let Some(id) = ctx.idmaps.enum_values.get(id) else {
                    return Err(InaccessibleEnumValue);
                };

                SchemaInputValueRecord::EnumValue(id)
            }
            Value::Object(fields) => {
                let ids = self.reserve_map(fields.len());
                for ((name, value), id) in fields.into_vec().into_iter().zip(ids) {
                    self[id] = (name.into(), self.ingest_arbitrary_federated_value(ctx, value)?);
                }
                self[ids].sort_unstable_by(|(left_key, _), (right_key, _)| {
                    ctx.strings.get_by_id(*left_key).cmp(&ctx.strings.get_by_id(*right_key))
                });
                SchemaInputValueRecord::Map(ids)
            }
            Value::List(list) => {
                let ids = self.reserve_list(list.len());
                for (value, id) in list.into_vec().into_iter().zip(ids) {
                    let value = self.ingest_arbitrary_federated_value(ctx, value)?;
                    self[id] = value;
                }
                SchemaInputValueRecord::List(ids)
            }
        })
    }
}
