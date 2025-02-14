use std::cmp::Ordering;

use schema::EntityDefinition;
use serde::ser::{Error, SerializeMap};

use super::{ForFieldSet, ResponseObjectView, ResponseValueView, WithExtraFields};
use crate::{prepare::RequiredFieldSet, response::ResponseValue};

impl serde::Serialize for ResponseObjectView<'_, WithExtraFields<'_>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(
            self.view.requirements.len() + self.view.extra_constant_fields.len(),
        ))?;

        for (name, value) in self.view.extra_constant_fields {
            map.serialize_entry(name, value)?
        }

        for selection in self.view.requirements.iter() {
            let field = selection.data_field();
            let key = field.definition().name();
            let value = ResponseValueView {
                ctx: self.ctx,
                value: self
                    .response_object
                    .find_by_response_key(field.response_key)
                    .ok_or_else(|| S::Error::custom(format!("Could not retrieve field {key}",)))?,
                view: selection.subselection(),
            };
            map.serialize_entry(key, &value)?;
        }

        map.end()
    }
}

impl serde::Serialize for ResponseObjectView<'_, RequiredFieldSet<'_>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.view.len()))?;
        for selection in self.view.iter() {
            let field = selection.data_field();
            let key = field.definition().name();
            let value = match self.response_object.find_by_response_key(field.response_key) {
                Some(value) => value,
                None => {
                    // If this field doesn't match the actual response object, meaning this field
                    // was in a fragment that doesn't apply to this object, we can safely skip it.
                    if let Some(definition_id) = self.response_object.definition_id {
                        match field.definition().parent_entity() {
                            EntityDefinition::Interface(inf) => {
                                if inf.possible_type_ids.binary_search(&definition_id).is_err() {
                                    continue;
                                }
                            }
                            EntityDefinition::Object(obj) => {
                                if obj.id != definition_id {
                                    continue;
                                }
                            }
                        }
                    }

                    return Err(S::Error::custom(format_args!(
                        "Could not retrieve field {}.{key}",
                        field.definition().parent_entity().name(),
                    )));
                }
            };
            let value = ResponseValueView {
                ctx: self.ctx,
                value,
                view: selection.subselection(),
            };
            map.serialize_entry(key, &value)?;
        }

        map.end()
    }
}

impl serde::Serialize for ResponseObjectView<'_, ForFieldSet<'_>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.view.field_set.len()))?;
        let mut f = 0;
        let mut r = 0;

        'field_set: while let Some(field_set_item) = self.view.field_set.get(f) {
            while let Some(selection) = self.view.requirements.get(r) {
                match field_set_item.field_id.cmp(&selection.matching_field_id) {
                    Ordering::Less => unreachable!("RequiredFieldSet should contain all requirements."),
                    Ordering::Equal => {
                        let field = selection.data_field();
                        let key = field.definition().name();
                        let value = match self.response_object.find_by_response_key(field.response_key) {
                            Some(value) => value,
                            None => {
                                // If this field doesn't match the actual response object, meaning this field
                                // was in a fragment that doesn't apply to this object, we can safely skip it.
                                if let Some(definition_id) = self.response_object.definition_id {
                                    match field.definition().parent_entity() {
                                        EntityDefinition::Interface(inf) => {
                                            if inf.possible_type_ids.binary_search(&definition_id).is_err() {
                                                f += 1;
                                                r += 1;
                                                continue 'field_set;
                                            }
                                        }
                                        EntityDefinition::Object(obj) => {
                                            if obj.id != definition_id {
                                                f += 1;
                                                r += 1;
                                                continue 'field_set;
                                            }
                                        }
                                    }
                                }

                                return Err(S::Error::custom(format_args!(
                                    "Could not retrieve field {}.{key}",
                                    field.definition().parent_entity().name(),
                                )));
                            }
                        };
                        let value = ResponseValueView {
                            ctx: self.ctx,
                            value,
                            view: ForFieldSet {
                                requirements: selection.subselection(),
                                field_set: &field_set_item.subselection_record,
                            },
                        };
                        map.serialize_entry(key, &value)?;

                        f += 1;
                        r += 1;
                        continue 'field_set;
                    }
                    Ordering::Greater => {
                        r += 1;
                    }
                }
            }
            unreachable!("RequiredFieldSet should contain all requirements.")
        }

        map.end()
    }
}

impl<View> serde::Serialize for ResponseValueView<'_, View>
where
    for<'a> ResponseObjectView<'a, View>: serde::Serialize,
    View: Copy,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.value {
            ResponseValue::Null => serializer.serialize_none(),
            ResponseValue::Inaccessible { id } => ResponseValueView {
                ctx: self.ctx,
                value: &self.ctx.response.data_parts[*id],
                view: self.view,
            }
            .serialize(serializer),
            ResponseValue::Boolean { value, .. } => value.serialize(serializer),
            ResponseValue::Int { value, .. } => value.serialize(serializer),
            ResponseValue::Float { value, .. } => value.serialize(serializer),
            ResponseValue::String { value, .. } => value.serialize(serializer),
            ResponseValue::StringId { id, .. } => self.ctx.response.schema[*id].serialize(serializer),
            ResponseValue::BigInt { value, .. } => value.serialize(serializer),
            &ResponseValue::List { id, .. } => {
                let values = &self.ctx.response.data_parts[id];
                serializer.collect_seq(values.iter().map(|value| ResponseValueView {
                    ctx: self.ctx,
                    value,
                    view: self.view,
                }))
            }
            &ResponseValue::Object { id, .. } => ResponseObjectView {
                ctx: self.ctx,
                response_object: &self.ctx.response.data_parts[id],
                view: self.view,
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
                            view: self.view,
                        },
                    )
                }))
            }
        }
    }
}
