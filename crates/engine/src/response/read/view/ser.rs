use std::cmp::Ordering;

use schema::{EntityDefinition, KeyValueInjectionRecord, ValueInjection};
use serde::ser::{Error, SerializeMap};
use walker::Walk as _;

use super::{ForFieldSet, ForInjection, ParentObjectsView, ResponseObjectView, ResponseValueView, WithExtraFields};
use crate::{prepare::RequiredFieldSet, response::ResponseValue};

impl<'a, View: Copy> serde::Serialize for ParentObjectsView<'a, View>
where
    ResponseObjectView<'a, View>: serde::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_seq(self.iter())
    }
}

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
                    .ok_or_else(|| S::Error::custom(format_args!("Could not retrieve field {key}",)))?,
                view: ForwardView(selection.subselection()),
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
                view: ForwardView(selection.subselection()),
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
        let mut r = 0;

        'field_set: for field_set_item in self.view.field_set {
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
                                                r += 1;
                                                continue 'field_set;
                                            }
                                        }
                                        EntityDefinition::Object(obj) => {
                                            if obj.id != definition_id {
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
                            view: ForwardView(ForFieldSet {
                                requirements: selection.subselection(),
                                field_set: &field_set_item.subselection_record,
                            }),
                        };
                        map.serialize_entry(key, &value)?;

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

#[derive(Debug, Clone, Copy)]
struct ForwardView<V>(V);

impl<'a, View> serde::Serialize for ResponseValueView<'a, ForwardView<View>>
where
    ResponseObjectView<'a, View>: serde::Serialize,
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
            ResponseValue::I64 { value, .. } => value.serialize(serializer),
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
                view: self.view.0,
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

impl serde::Serialize for ResponseValueView<'_, Option<ForInjection<'_>>> {
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
            &ResponseValue::List { id, .. } => {
                let values = &self.ctx.response.data_parts[id];
                serializer.collect_seq(values.iter().map(|value| ResponseValueView {
                    ctx: self.ctx,
                    value,
                    view: self.view,
                }))
            }
            ResponseValue::Unexpected => Err(S::Error::custom("Unexpected value")),

            ResponseValue::Object { id, .. } => match self.view {
                Some(view) => ResponseObjectView {
                    ctx: self.ctx,
                    response_object: &self.ctx.response.data_parts[*id],
                    view,
                }
                .serialize(serializer),
                None => {
                    unreachable!("Is not a scalar");
                }
            },
            ResponseValue::Boolean { value, .. } => {
                debug_assert!(self.view.is_none());
                value.serialize(serializer)
            }
            ResponseValue::Int { value, .. } => {
                debug_assert!(self.view.is_none());
                value.serialize(serializer)
            }
            ResponseValue::Float { value, .. } => {
                debug_assert!(self.view.is_none());
                value.serialize(serializer)
            }
            ResponseValue::String { value, .. } => {
                debug_assert!(self.view.is_none());
                value.serialize(serializer)
            }
            ResponseValue::StringId { id, .. } => {
                debug_assert!(self.view.is_none());
                self.ctx.response.schema[*id].serialize(serializer)
            }
            ResponseValue::I64 { value, .. } => {
                debug_assert!(self.view.is_none());
                value.serialize(serializer)
            }
            ResponseValue::U64 { value } => {
                debug_assert!(self.view.is_none());
                value.serialize(serializer)
            }
            ResponseValue::Map { id } => {
                debug_assert!(self.view.is_none());
                serializer.collect_map(self.ctx.response.data_parts[*id].iter().map(|(key, value)| {
                    (
                        key.as_str(),
                        ResponseValueView {
                            ctx: self.ctx,
                            value,
                            view: None,
                        },
                    )
                }))
            }
        }
    }
}

impl serde::Serialize for ResponseObjectView<'_, ForInjection<'_>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let schema = self.ctx.schema();
        match self.view.injection {
            ValueInjection::Const(id) => id.walk(schema).serialize(serializer),
            ValueInjection::Select { field_id, next } => {
                let Some(selection) = self
                    .view
                    .requirements
                    .iter()
                    .find(|field| field.matching_field_id == field_id)
                else {
                    let def = field_id.walk(schema).definition();
                    return Err(S::Error::custom(format_args!(
                        "Could not retrieve field {}.{}",
                        def.parent_entity().name(),
                        def.name()
                    )));
                };

                let field = self
                    .response_object
                    .find_by_response_key(selection.data_field().response_key)
                    .ok_or_else(|| S::Error::custom(format_args!("Could not retrieve field {field_id}")))?;
                ResponseValueView {
                    ctx: self.ctx,
                    value: field,
                    view: next.map(|id| ForInjection {
                        injection: schema[id],
                        requirements: selection.subselection(),
                    }),
                }
                .serialize(serializer)
            }
            ValueInjection::Object(ids) => {
                let injections = &schema[ids];

                let mut map = serializer.serialize_map(Some(injections.len()))?;
                let mut r = 0;

                'injections: for &KeyValueInjectionRecord { key_id, value } in injections {
                    match value {
                        ValueInjection::Select { field_id, next } => {
                            while let Some(selection) = self.view.requirements.get(r) {
                                match field_id.cmp(&selection.matching_field_id) {
                                    Ordering::Less => unreachable!(
                                        "RequiredFieldSet should contain all requirements, tried to load {}",
                                        selection.data_field().definition().name()
                                    ),
                                    Ordering::Equal => {
                                        let field = selection.data_field();
                                        let value = match self.response_object.find_by_response_key(field.response_key)
                                        {
                                            Some(value) => value,
                                            None => {
                                                // If this field doesn't match the actual response object, meaning this field
                                                // was in a fragment that doesn't apply to this object, we can safely skip it.
                                                if let Some(definition_id) = self.response_object.definition_id {
                                                    match field.definition().parent_entity() {
                                                        EntityDefinition::Interface(inf) => {
                                                            if inf
                                                                .possible_type_ids
                                                                .binary_search(&definition_id)
                                                                .is_err()
                                                            {
                                                                continue 'injections;
                                                            }
                                                        }
                                                        EntityDefinition::Object(obj) => {
                                                            if obj.id != definition_id {
                                                                continue 'injections;
                                                            }
                                                        }
                                                    }
                                                }

                                                return Err(S::Error::custom(format_args!(
                                                    "Could not retrieve field {}.{}",
                                                    field.definition().parent_entity().name(),
                                                    field.definition().name()
                                                )));
                                            }
                                        };
                                        let value = ResponseValueView {
                                            ctx: self.ctx,
                                            value,
                                            view: next.map(|id| ForInjection {
                                                injection: schema[id],
                                                requirements: selection.subselection(),
                                            }),
                                        };
                                        map.serialize_entry(&schema[key_id], &value)?;

                                        continue 'injections;
                                    }
                                    Ordering::Greater => {
                                        r += 1;
                                    }
                                }
                            }
                        }
                        injection => {
                            map.serialize_entry(
                                &schema[key_id],
                                &Self {
                                    ctx: self.ctx,
                                    response_object: self.response_object,
                                    view: ForInjection {
                                        injection,
                                        requirements: self.view.requirements,
                                    },
                                },
                            )?;
                        }
                    }
                }

                map.end()
            }
        }
    }
}
