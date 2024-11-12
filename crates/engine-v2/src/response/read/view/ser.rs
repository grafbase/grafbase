use serde::ser::SerializeMap;

use super::{
    ResponseObjectView, ResponseObjectViewWithExtraFields, ResponseObjectsView, ResponseObjectsViewWithExtraFields,
    ResponseValueView,
};
use crate::response::{ResponseListId, ResponseObjectId, ResponseValue, NULL};

impl<'a> serde::Serialize for ResponseObjectsView<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_seq(self.clone())
    }
}

impl<'a> serde::Serialize for ResponseObjectsViewWithExtraFields<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_seq(self.iter())
    }
}

impl<'a> serde::Serialize for ResponseObjectViewWithExtraFields<'a> {
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
                value: self.response_object.find_required_field(selection.id).unwrap_or(&NULL),
                selection_set: &selection.subselection_record,
            };
            map.serialize_entry(key, &value)?;
        }

        map.end()
    }
}

impl<'a> serde::Serialize for ResponseObjectView<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_map(self.selection_set.iter().map(|selection| {
            let key = self.ctx.schema[selection.alias_id].as_str();
            let value = ResponseValueView {
                ctx: self.ctx,
                value: self.response_object.find_required_field(selection.id).unwrap_or(&NULL),
                selection_set: &selection.subselection_record,
            };

            (key, value)
        }))
    }
}

impl<'a> serde::Serialize for ResponseValueView<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.value {
            ResponseValue::Null => serializer.serialize_none(),
            ResponseValue::Boolean { value, .. } => value.serialize(serializer),
            ResponseValue::Int { value, .. } => value.serialize(serializer),
            ResponseValue::Float { value, .. } => value.serialize(serializer),
            ResponseValue::String { value, .. } => value.serialize(serializer),
            ResponseValue::StringId { id, .. } => self.ctx.schema[*id].serialize(serializer),
            ResponseValue::BigInt { value, .. } => value.serialize(serializer),
            &ResponseValue::List {
                part_id,
                offset,
                length,
                ..
            } => {
                let values = &self.ctx.response[ResponseListId {
                    part_id,
                    offset,
                    length,
                }];
                serializer.collect_seq(values.iter().map(|value| ResponseValueView {
                    ctx: self.ctx,
                    value,
                    selection_set: self.selection_set,
                }))
            }
            &ResponseValue::Object { part_id, index, .. } => ResponseObjectView {
                ctx: self.ctx,
                response_object: &self.ctx.response[ResponseObjectId { part_id, index }],
                selection_set: self.selection_set,
            }
            .serialize(serializer),
            ResponseValue::Json { value, .. } => value.serialize(serializer),
        }
    }
}
