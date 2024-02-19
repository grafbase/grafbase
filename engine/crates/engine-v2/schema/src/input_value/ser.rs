use serde::ser::{SerializeMap, SerializeSeq};

use crate::{RawInputValue, RawInputValueWalker};

use super::RawInputValuesContext;

impl<'ctx, Ctx> serde::Serialize for RawInputValueWalker<'ctx, Ctx>
where
    Ctx: RawInputValuesContext<'ctx>,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.value {
            RawInputValue::Undefined => serializer.serialize_unit(),
            RawInputValue::Null => serializer.serialize_none(),
            RawInputValue::String(s) | RawInputValue::UnknownEnumValue(s) => self.ctx.get_str(s).serialize(serializer),
            RawInputValue::EnumValue(id) => self.ctx.schema_walker().walk(*id).name().serialize(serializer),
            RawInputValue::Int(n) => n.serialize(serializer),
            RawInputValue::BigInt(n) => n.serialize(serializer),
            RawInputValue::Float(f) => f.serialize(serializer),
            RawInputValue::U64(n) => n.serialize(serializer),
            RawInputValue::Boolean(b) => b.serialize(serializer),
            &RawInputValue::InputObject(ids) => {
                let mut map = serializer.serialize_map(Some(ids.len()))?;
                for (input_value_definition_id, value) in &self.ctx.input_values()[ids] {
                    let value = self.walk(value);
                    if !value.is_undefined() {
                        map.serialize_key(self.ctx.schema_walker().walk(*input_value_definition_id).name())?;
                        map.serialize_value(&value)?;
                    }
                }
                map.end()
            }
            &RawInputValue::List(ids) => {
                let mut seq = serializer.serialize_seq(Some(ids.len()))?;
                for value in &self.ctx.input_values()[ids] {
                    seq.serialize_element(&self.walk(value))?;
                }
                seq.end()
            }
            &RawInputValue::Map(ids) => {
                let mut map = serializer.serialize_map(Some(ids.len()))?;
                for (key, value) in &self.ctx.input_values()[ids] {
                    let value = self.walk(value);
                    if !value.is_undefined() {
                        map.serialize_key(self.ctx.get_str(key))?;
                        map.serialize_value(&value)?;
                    }
                }
                map.end()
            }
            RawInputValue::Ref(id) => self.ctx.walk(*id).serialize(serializer),
            RawInputValue::SchemaRef(id) => self.ctx.schema_walk(*id).serialize(serializer),
        }
    }
}
