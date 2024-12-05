use serde::ser::{Error, SerializeMap};
use walker::Walk;

use super::{ResponseObjectView, ResponseObjectViewWithExtraFields, ResponseValueView};
use crate::response::ResponseValue;

impl serde::Serialize for ResponseObjectViewWithExtraFields<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.selection_set.len() + self.extra_constant_fields.len()))?;

        for (name, value) in self.extra_constant_fields {
            map.serialize_entry(name, value)?
        }

        for selection in self.selection_set {
            let key = self.ctx.schema[selection.alias_id].as_str();
            let value = ResponseValueView {
                ctx: self.ctx,
                value: self.response_object.find_required_field(selection.id).ok_or_else(|| {
                    S::Error::custom(format!(
                        "Could not retrieve field {}",
                        selection.id.walk(self.ctx.schema).definition().name()
                    ))
                })?,
                selection_set: &selection.subselection_record,
            };
            map.serialize_entry(key, &value)?;
        }

        map.end()
    }
}

impl serde::Serialize for ResponseObjectView<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.selection_set.len()))?;

        for selection in self.selection_set {
            let key = self.ctx.schema[selection.alias_id].as_str();
            let value = ResponseValueView {
                ctx: self.ctx,
                value: self.response_object.find_required_field(selection.id).ok_or_else(|| {
                    S::Error::custom(format!(
                        "Could not retrieve field {}",
                        selection.id.walk(self.ctx.schema).definition().name()
                    ))
                })?,
                selection_set: &selection.subselection_record,
            };
            map.serialize_entry(key, &value)?;
        }

        map.end()
    }
}

impl serde::Serialize for ResponseValueView<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.value {
            ResponseValue::Null => serializer.serialize_none(),
            ResponseValue::Inaccessible { id } => ResponseValueView {
                ctx: self.ctx,
                value: &self.ctx.response.data_parts[*id],
                selection_set: self.selection_set,
            }
            .serialize(serializer),
            ResponseValue::Boolean { value, .. } => value.serialize(serializer),
            ResponseValue::Int { value, .. } => value.serialize(serializer),
            ResponseValue::Float { value, .. } => value.serialize(serializer),
            ResponseValue::String { value, .. } => value.serialize(serializer),
            ResponseValue::StringId { id, .. } => self.ctx.schema[*id].serialize(serializer),
            ResponseValue::BigInt { value, .. } => value.serialize(serializer),
            &ResponseValue::List { id, .. } => {
                let values = &self.ctx.response.data_parts[id];
                serializer.collect_seq(values.iter().map(|value| ResponseValueView {
                    ctx: self.ctx,
                    value,
                    selection_set: self.selection_set,
                }))
            }
            &ResponseValue::Object { id, .. } => ResponseObjectView {
                ctx: self.ctx,
                response_object: &self.ctx.response.data_parts[id],
                selection_set: self.selection_set,
            }
            .serialize(serializer),
            ResponseValue::Unexpected => Err(S::Error::custom("Unexpected value")),
            ResponseValue::U64 { value } => value.serialize(serializer),
            ResponseValue::Map { id } => {
                serializer.collect_map(self.ctx.response.data_parts[*id].iter().map(|(key, value)| {
                    (
                        key.as_str(),
                        ResponseValueView {
                            ctx: self.ctx,
                            value,
                            selection_set: self.selection_set,
                        },
                    )
                }))
            }
        }
    }
}
