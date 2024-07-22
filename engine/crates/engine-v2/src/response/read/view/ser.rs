use serde::ser::{SerializeMap, SerializeSeq};

use super::{ResponseObjectView, ResponseObjectWithExtraFieldsWalker, ResponseValueWalker};
use crate::response::{ResponseListId, ResponseObjectId, ResponseValue};

impl<'a> serde::Serialize for super::ResponseObjectsView<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(None)?;
        for item in self.clone() {
            seq.serialize_element(&item)?;
        }
        seq.end()
    }
}

impl<'a> serde::Serialize for super::ResponseObjectsViewWithExtraFields<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(None)?;
        for item in self.iter() {
            seq.serialize_element(&item)?;
        }
        seq.end()
    }
}

impl<'a> serde::Serialize for ResponseObjectWithExtraFieldsWalker<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.selection_set.len() + self.extra_constant_fields.len()))?;
        for (name, value) in self.extra_constant_fields {
            map.serialize_key(name)?;
            map.serialize_value(value)?;
        }
        for selection in &self.ctx.response_views[self.selection_set] {
            map.serialize_key(&self.ctx.schema[selection.name])?;
            if let Some(value) = self.response_object.find_required_field(selection.id) {
                map.serialize_value(&ResponseValueWalker {
                    ctx: self.ctx,
                    value,
                    selection_set: self.selection_set,
                })?;
            } else {
                map.serialize_value(&None::<()>)?
            }
        }

        map.end()
    }
}

impl<'a> serde::Serialize for ResponseObjectView<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.selection_set.len()))?;
        for selection in &self.ctx.response_views[self.selection_set] {
            map.serialize_key(&self.ctx.schema[selection.name])?;
            if let Some(value) = self.response_object.find_required_field(selection.id) {
                map.serialize_value(&ResponseValueWalker {
                    ctx: self.ctx,
                    value,
                    selection_set: selection.subselection,
                })?;
            } else {
                map.serialize_value(&None::<()>)?
            }
        }

        map.end()
    }
}

impl<'a> serde::Serialize for ResponseValueWalker<'a> {
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
                let mut seq = serializer.serialize_seq(Some(values.len()))?;
                for value in values {
                    seq.serialize_element(&ResponseValueWalker {
                        ctx: self.ctx,
                        value,
                        selection_set: self.selection_set,
                    })?;
                }
                seq.end()
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
