use serde::de::{DeserializeSeed, Error, Unexpected, Visitor};
use walker::Walk;

use crate::{prepare::FieldShapeRecord, response::GraphqlError};

use super::super::SeedState;

#[derive(Clone, Copy)]
pub(crate) struct NonNullIntSeed<'ctx, 'parent, 'state> {
    pub state: &'state SeedState<'ctx, 'parent>,
    pub field: &'ctx FieldShapeRecord,
}

impl<'de> DeserializeSeed<'de> for NonNullIntSeed<'_, '_, '_> {
    // We return Result<i32, ()> so that we know whether we encountered an invalid value or not,
    // without failing de-serialization
    type Value = Result<i32, ()>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }
}

impl NonNullIntSeed<'_, '_, '_> {
    fn unexpected_type(&self, value: Unexpected<'_>) {
        tracing::error!(
            "invalid type: {}, expected an Int value at '{}'",
            value,
            self.state.display_path()
        );

        if self.state.should_report_error_for(self.field) {
            let mut resp = self.state.response.borrow_mut();
            let path = self.state.path();
            resp.propagate_null(&path);
            resp.errors.push(
                GraphqlError::invalid_subgraph_response()
                    .with_path(path)
                    .with_location(self.field.id.walk(self.state).location()),
            );
        }
    }
}

impl<'de> Visitor<'de> for NonNullIntSeed<'_, '_, '_> {
    type Value = Result<i32, ()>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("an Int value")
    }

    fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(Ok(v as i32))
    }

    fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(Ok(v as i32))
    }

    fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(Ok(v))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        if let Ok(value) = i32::try_from(v) {
            Ok(Ok(value))
        } else {
            self.unexpected_type(Unexpected::Signed(v));
            Ok(Err(()))
        }
    }

    fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
    where
        E: Error,
    {
        if let Ok(value) = i32::try_from(v) {
            Ok(Ok(value))
        } else {
            self.unexpected_type(Unexpected::Other(&format!("integer {v}")));
            Ok(Err(()))
        }
    }

    fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(Ok(v as i32))
    }

    fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(Ok(v as i32))
    }

    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
    where
        E: Error,
    {
        if let Ok(value) = i32::try_from(v) {
            Ok(Ok(value))
        } else {
            self.unexpected_type(Unexpected::Unsigned(v.into()));
            Ok(Err(()))
        }
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        if let Ok(value) = i32::try_from(v) {
            Ok(Ok(value))
        } else {
            self.unexpected_type(Unexpected::Unsigned(v));
            Ok(Err(()))
        }
    }

    fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
    where
        E: Error,
    {
        if let Ok(value) = i32::try_from(v) {
            Ok(Ok(value))
        } else {
            self.unexpected_type(Unexpected::Other(&format!("integer {v}")));
            Ok(Err(()))
        }
    }

    fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
    where
        E: Error,
    {
        if can_coerce_f32_to_int(v) {
            Ok(Ok(v as i32))
        } else {
            self.unexpected_type(Unexpected::Float(v as f64));
            Ok(Err(()))
        }
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        if can_coerce_f64_to_int(v) {
            Ok(Ok(v as i32))
        } else {
            self.unexpected_type(Unexpected::Float(v));
            Ok(Err(()))
        }
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: Error,
    {
        self.unexpected_type(Unexpected::Bool(v));
        Ok(Err(()))
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        self.unexpected_type(Unexpected::Str(v));
        Ok(Err(()))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        self.unexpected_type(Unexpected::Str(v));
        Ok(Err(()))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: Error,
    {
        self.unexpected_type(Unexpected::Str(&v));
        Ok(Err(()))
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: Error,
    {
        self.unexpected_type(Unexpected::Bytes(v));
        Ok(Err(()))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: Error,
    {
        self.unexpected_type(Unexpected::Option);
        Ok(Err(()))
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: Error,
    {
        self.unexpected_type(Unexpected::Unit);
        Ok(Err(()))
    }

    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }

    fn visit_seq<A>(self, _seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        self.unexpected_type(Unexpected::Seq);
        Ok(Err(()))
    }

    fn visit_map<A>(self, _map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        self.unexpected_type(Unexpected::Map);
        Ok(Err(()))
    }

    fn visit_enum<A>(self, _data: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::EnumAccess<'de>,
    {
        self.unexpected_type(Unexpected::Enum);
        Ok(Err(()))
    }
}

fn can_coerce_f32_to_int(float: f32) -> bool {
    float.floor() == float && float < (i32::MAX as f32)
}

fn can_coerce_f64_to_int(float: f64) -> bool {
    float.floor() == float && float < (i32::MAX as f64)
}
