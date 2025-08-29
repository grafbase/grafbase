use std::fmt;

use serde::{
    Deserializer,
    de::{DeserializeSeed, IgnoredAny, SeqAccess, Unexpected, Visitor},
};
use walker::Walk;

use super::SeedState;
use crate::{
    prepare::FieldShapeRecord,
    response::{GraphqlError, ResponseValue, ResponseValueId},
};

pub(super) struct ListSeed<'ctx, 'parent, 'state, Seed> {
    pub field: &'ctx FieldShapeRecord,
    pub state: &'state SeedState<'ctx, 'parent>,
    pub seed: &'state Seed,
    pub is_required: bool,
    pub element_is_nullable: bool,
}

impl<'de, Seed> DeserializeSeed<'de> for ListSeed<'_, '_, '_, Seed>
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

impl<'de, Seed> ListSeed<'_, '_, '_, Seed>
where
    Seed: Clone + DeserializeSeed<'de, Value = ResponseValue>,
{
    fn unexpected_type(&self, value: Unexpected<'_>) -> <Self as Visitor<'de>>::Value {
        tracing::error!(
            "invalid type: {}, expected a list at path '{}'",
            value,
            self.state.display_path()
        );

        if self.state.should_report_error_for(self.field) {
            let mut resp = self.state.response.borrow_mut();
            let path = self.state.path();
            // If not required, we don't need to propagate as Unexpected is equivalent to
            // null for users.
            if self.is_required {
                resp.propagate_null(&path);
            }
            resp.errors.push(
                GraphqlError::invalid_subgraph_response()
                    .with_path(path)
                    .with_location(self.field.id.walk(self.state).location()),
            );
        }

        ResponseValue::Unexpected
    }
}

impl<'de, Seed> Visitor<'de> for ListSeed<'_, '_, '_, Seed>
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
            field,
            state,
            seed,
            element_is_nullable,
            ..
        } = self;

        let mut index: u32 = 0;
        let (list_id, mut list) = state.response.borrow_mut().data.take_next_list();
        if let Some(size_hint) = seq.size_hint() {
            list.reserve(size_hint);
        }
        let offset = list.len() as u32;

        loop {
            state
                .local_path_mut()
                .push(ResponseValueId::index(list_id, offset + index, element_is_nullable));
            let result = seq.next_element_seed(seed.clone());
            state.local_path_mut().pop();
            match result {
                Ok(Some(value)) => {
                    list.push(value);
                    index += 1;
                }
                Ok(None) => {
                    break;
                }
                Err(err) => {
                    let mut resp = state.response.borrow_mut();
                    if !state.bubbling_up_deser_error.replace(true) && state.should_report_error_for(field) {
                        tracing::error!(
                            "Deserialization failure of subgraph response at path '{}': {err}",
                            self.state.display_path()
                        );
                        let path = state.path();
                        resp.propagate_null(&path);
                        resp.errors.push(
                            GraphqlError::invalid_subgraph_response()
                                .with_path((path, index))
                                .with_location(field.id.walk(state).location()),
                        );
                    }

                    resp.data.restore_list(list_id, list);
                    return Err(err);
                }
            }
        }
        let limit = index;

        state.response.borrow_mut().data.restore_list(list_id, list);
        Ok(ResponseValue::List {
            id: list_id,
            offset,
            limit,
        })
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

    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
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
