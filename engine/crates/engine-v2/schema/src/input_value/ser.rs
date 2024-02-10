use serde::ser::{SerializeMap, SerializeSeq};

use crate::{IdRange, InputKeyValueId, InputObjectFieldValueId, InputValue, InputValueId};

use super::InputValuesContext;

pub(super) struct SerializableInputValue<'ctx, Str, Ctx> {
    pub ctx: Ctx,
    pub value: &'ctx InputValue<Str>,
}

impl<'ctx, Str, Ctx> serde::Serialize for SerializableInputValue<'ctx, Str, Ctx>
where
    Ctx: InputValuesContext<'ctx, Str>,
    Str: 'ctx,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.value {
            InputValue::Null => serializer.serialize_unit(),
            InputValue::String(s) | InputValue::UnknownEnumValue(s) => self.ctx.get_str(s).serialize(serializer),
            InputValue::EnumValue(id) => self.ctx.schema_walker().walk(*id).name().serialize(serializer),
            InputValue::Int(n) => n.serialize(serializer),
            InputValue::BigInt(n) => n.serialize(serializer),
            InputValue::Float(f) => f.serialize(serializer),
            InputValue::U64(n) => n.serialize(serializer),
            InputValue::Boolean(b) => b.serialize(serializer),
            &InputValue::InputObject(input_fields) => SerializableInputObject {
                ctx: self.ctx,
                input_fields,
            }
            .serialize(serializer),
            &InputValue::List(list) => SerializableList { ctx: self.ctx, list }.serialize(serializer),
            &InputValue::Map(fields) => SerializableMap { ctx: self.ctx, fields }.serialize(serializer),
        }
    }
}

struct SerializableMap<Str, Ctx> {
    ctx: Ctx,
    fields: IdRange<InputKeyValueId<Str>>,
}

impl<'ctx, Str, Ctx> serde::Serialize for SerializableMap<Str, Ctx>
where
    Ctx: InputValuesContext<'ctx, Str>,
    Str: 'ctx,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.fields.len()))?;
        for (key, value) in &self.ctx.input_values()[self.fields] {
            map.serialize_key(self.ctx.get_str(key))?;
            map.serialize_value(&SerializableInputValue { ctx: self.ctx, value })?;
        }

        map.end()
    }
}
struct SerializableInputObject<Str, Ctx> {
    ctx: Ctx,
    input_fields: IdRange<InputObjectFieldValueId<Str>>,
}

impl<'ctx, Str, Ctx> serde::Serialize for SerializableInputObject<Str, Ctx>
where
    Ctx: InputValuesContext<'ctx, Str>,
    Str: 'ctx,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.input_fields.len()))?;
        for (input_value_definition_id, value) in &self.ctx.input_values()[self.input_fields] {
            map.serialize_key(self.ctx.schema_walker().walk(*input_value_definition_id).name())?;
            map.serialize_value(&SerializableInputValue { ctx: self.ctx, value })?;
        }

        map.end()
    }
}

struct SerializableList<Str, Ctx> {
    ctx: Ctx,
    list: IdRange<InputValueId<Str>>,
}

impl<'ctx, Str, Ctx> serde::Serialize for SerializableList<Str, Ctx>
where
    Ctx: InputValuesContext<'ctx, Str>,
    Str: 'ctx,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.list.len()))?;
        for value in &self.ctx.input_values()[self.list] {
            seq.serialize_element(&SerializableInputValue { ctx: self.ctx, value })?;
        }
        seq.end()
    }
}
