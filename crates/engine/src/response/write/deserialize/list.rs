use std::fmt;

use serde::{
    Deserializer,
    de::{DeserializeSeed, IgnoredAny, SeqAccess, Unexpected, Visitor},
};
use walker::Walk;

use super::SeedContext;
use crate::{
    prepare::FieldShapeRecord,
    response::{GraphqlError, ResponseValue, ResponseValueId},
};

pub(super) struct ListSeed<'ctx, 'parent, Seed> {
    pub ctx: &'parent SeedContext<'ctx>,
    pub parent_field: &'ctx FieldShapeRecord,
    pub seed: &'parent Seed,
    pub is_required: bool,
    pub element_is_nullable: bool,
}

impl<'de, Seed> DeserializeSeed<'de> for ListSeed<'_, '_, Seed>
where
    Seed: Clone + DeserializeSeed<'de, Value = ResponseValue>,
{
    type Value = ResponseValue;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }
}

impl<'de, Seed> ListSeed<'_, '_, Seed>
where
    Seed: Clone + DeserializeSeed<'de, Value = ResponseValue>,
{
    fn unexpected_type(&self, value: Unexpected<'_>) -> <Self as Visitor<'de>>::Value {
        tracing::error!(
            "invalid type: {}, expected a list at path '{}'",
            value,
            self.ctx.display_path()
        );

        if self.parent_field.key.query_position.is_some() {
            let mut resp = self.ctx.response.borrow_mut();
            let path = self.ctx.path();
            // If not required, we don't need to propagate as Unexpected is equivalent to
            // null for users.
            if self.is_required {
                resp.propagate_null(&path);
            }
            resp.push_error(
                GraphqlError::invalid_subgraph_response()
                    .with_path(path)
                    .with_location(self.parent_field.id.walk(self.ctx).location()),
            );
        }

        ResponseValue::Unexpected
    }
}

impl<'de, Seed> Visitor<'de> for ListSeed<'_, '_, Seed>
where
    Seed: Clone + DeserializeSeed<'de, Value = ResponseValue>,
{
    type Value = ResponseValue;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("any value?")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let ListSeed {
            ctx,
            parent_field,
            seed,
            element_is_nullable,
            ..
        } = self;

        let mut index: u32 = 0;
        let list_id = ctx.response.borrow_mut().data.reserve_list_id();
        let mut list = Vec::new();
        if let Some(size_hint) = seq.size_hint() {
            list.reserve(size_hint);
        }

        loop {
            ctx.path_mut().push(ResponseValueId::Index {
                list_id,
                index,
                nullable: element_is_nullable,
            });
            let result = seq.next_element_seed(seed.clone());
            ctx.path_mut().pop();
            match result {
                Ok(Some(value)) => {
                    list.push(value);
                    index += 1;
                }
                Ok(None) => {
                    break;
                }
                Err(err) => {
                    if !ctx.bubbling_up_serde_error.get() && parent_field.key.query_position.is_some() {
                        ctx.bubbling_up_serde_error.set(true);
                        tracing::error!(
                            "Deserialization failure of subgraph response at path '{}': {err}",
                            self.ctx.display_path()
                        );
                        let mut resp = ctx.response.borrow_mut();
                        resp.propagate_null(&ctx.path());
                        resp.push_error(
                            GraphqlError::invalid_subgraph_response()
                                .with_path((ctx.path().as_ref(), index))
                                .with_location(parent_field.id.walk(ctx).location()),
                        );
                    }

                    return Err(err);
                }
            }
        }

        ctx.response.borrow_mut().data.put_list(list_id, list);
        Ok(list_id.into())
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(self.unexpected_type(Unexpected::Bool(v)))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(self.unexpected_type(Unexpected::Signed(v)))
    }

    fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(self.unexpected_type(Unexpected::Other(&format!("integer {v}"))))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(self.unexpected_type(Unexpected::Unsigned(v)))
    }

    fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(self.unexpected_type(Unexpected::Other(&format!("integer {v}"))))
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(self.unexpected_type(Unexpected::Float(v)))
    }

    fn visit_char<E>(self, v: char) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_str(v.encode_utf8(&mut [0u8; 4]))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(self.unexpected_type(Unexpected::Str(v)))
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(self.unexpected_type(Unexpected::Bytes(v)))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if self.is_required {
            Ok(self.unexpected_type(Unexpected::Option))
        } else {
            Ok(ResponseValue::Null)
        }
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if self.is_required {
            Ok(self.unexpected_type(Unexpected::Unit))
        } else {
            Ok(ResponseValue::Null)
        }
    }

    fn visit_newtype_struct<D>(self, _: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        // newtype_struct are used by custom deserializers to indicate that an error happened, but
        // was already treated.
        Ok(ResponseValue::Unexpected)
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        // Try discarding the rest of the map, we might be able to use other parts of
        // the response.
        while map.next_entry::<IgnoredAny, IgnoredAny>()?.is_some() {}
        Ok(self.unexpected_type(Unexpected::Map))
    }

    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::EnumAccess<'de>,
    {
        let _ = data.variant::<IgnoredAny>()?;
        Ok(self.unexpected_type(Unexpected::Enum))
    }
}
