use schema::ScalarType;
use serde::de::{DeserializeSeed, Error, IgnoredAny, Unexpected, Visitor};
use walker::Walk;

use crate::response::{FieldShapeRecord, GraphqlError, ResponseValue};

use super::SeedContext;

#[derive(Clone)]
pub(crate) struct ScalarTypeSeed<'ctx, 'seed> {
    pub ctx: &'seed SeedContext<'ctx>,
    pub parent_field: &'ctx FieldShapeRecord,
    pub is_required: bool,
    pub ty: ScalarType,
}

impl<'de> DeserializeSeed<'de> for ScalarTypeSeed<'_, '_> {
    type Value = ResponseValue;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }
}

impl ScalarTypeSeed<'_, '_> {
    fn unexpected_type(&self, value: Unexpected<'_>) -> <Self as Visitor<'_>>::Value {
        let expected = match &self.ty {
            ScalarType::String => "a String value",
            ScalarType::Float => "a Float value",
            ScalarType::Int => "an Int value",
            ScalarType::BigInt => "a BigInt value",
            ScalarType::Any => "a JSON value",
            ScalarType::Boolean => "a Boolean value",
        };
        tracing::error!(
            "invalid type: {}, expected {} at '{}'",
            value,
            expected,
            self.ctx.display_path()
        );

        if self.parent_field.key.query_position.is_some() {
            let mut resp = self.ctx.subgraph_response.borrow_mut();
            let path = self.ctx.path();
            // If not required, we don't need to propagate as Unexpected is equivalent to
            // null for users.
            if self.is_required {
                resp.propagate_null(&path);
            }
            resp.push_error(
                GraphqlError::invalid_subgraph_response()
                    .with_path(path)
                    .with_location(self.parent_field.id.walk(self.ctx).location),
            );
        }

        ResponseValue::Unexpected
    }
}

impl<'de> Visitor<'de> for ScalarTypeSeed<'_, '_> {
    type Value = ResponseValue;
    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("any value?")
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(match self.ty {
            ScalarType::Boolean | ScalarType::Any => v.into(),
            _ => self.unexpected_type(Unexpected::Bool(v)),
        })
    }

    fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(match self.ty {
            ScalarType::Int | ScalarType::Any => ResponseValue::Int { value: v as i32 },
            ScalarType::BigInt => ResponseValue::BigInt { value: v as i64 },
            ScalarType::Float => ResponseValue::Float { value: v as f64 },
            _ => self.unexpected_type(Unexpected::Signed(v.into())),
        })
    }

    fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(match self.ty {
            ScalarType::Int | ScalarType::Any => ResponseValue::Int { value: v as i32 },
            ScalarType::BigInt => ResponseValue::BigInt { value: v as i64 },
            ScalarType::Float => ResponseValue::Float { value: v as f64 },
            _ => self.unexpected_type(Unexpected::Signed(v.into())),
        })
    }

    fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(match self.ty {
            ScalarType::Int | ScalarType::Any => ResponseValue::Int { value: v },
            ScalarType::BigInt => ResponseValue::BigInt { value: v as i64 },
            ScalarType::Float => ResponseValue::Float { value: v as f64 },
            _ => self.unexpected_type(Unexpected::Signed(v.into())),
        })
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(match self.ty {
            ScalarType::Int => {
                if let Ok(value) = i32::try_from(v) {
                    ResponseValue::Int { value }
                } else {
                    self.unexpected_type(Unexpected::Signed(v))
                }
            }
            ScalarType::BigInt | ScalarType::Any => ResponseValue::BigInt { value: v },
            ScalarType::Float => ResponseValue::Float { value: v as f64 },
            _ => self.unexpected_type(Unexpected::Signed(v)),
        })
    }

    fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
    where
        E: Error,
    {
        match self.ty {
            ScalarType::Int => {
                if let Ok(value) = i32::try_from(v) {
                    return Ok(ResponseValue::Int { value });
                }
            }
            ScalarType::BigInt | ScalarType::Any => {
                if let Ok(value) = i64::try_from(v) {
                    return Ok(ResponseValue::BigInt { value });
                }
            }
            ScalarType::Float => return Ok(ResponseValue::Float { value: v as f64 }),
            _ => (),
        };

        Ok(self.unexpected_type(Unexpected::Other(&format!("integer {v}"))))
    }

    fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(match self.ty {
            ScalarType::Int | ScalarType::Any => ResponseValue::Int { value: v as i32 },
            ScalarType::BigInt => ResponseValue::BigInt { value: v as i64 },
            ScalarType::Float => ResponseValue::Float { value: v as f64 },
            _ => self.unexpected_type(Unexpected::Unsigned(v.into())),
        })
    }

    fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(match self.ty {
            ScalarType::Int | ScalarType::Any => ResponseValue::Int { value: v as i32 },
            ScalarType::BigInt => ResponseValue::BigInt { value: v as i64 },
            ScalarType::Float => ResponseValue::Float { value: v as f64 },
            _ => self.unexpected_type(Unexpected::Unsigned(v.into())),
        })
    }

    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(match self.ty {
            ScalarType::Int => {
                if let Ok(value) = i32::try_from(v) {
                    ResponseValue::Int { value }
                } else {
                    self.unexpected_type(Unexpected::Unsigned(v.into()))
                }
            }
            ScalarType::BigInt | ScalarType::Any => ResponseValue::BigInt { value: v as i64 },
            ScalarType::Float => ResponseValue::Float { value: v as f64 },
            _ => self.unexpected_type(Unexpected::Unsigned(v.into())),
        })
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        match self.ty {
            ScalarType::Int => {
                if let Ok(value) = i32::try_from(v) {
                    return Ok(ResponseValue::Int { value });
                }
            }
            ScalarType::BigInt => {
                if let Ok(value) = i64::try_from(v) {
                    return Ok(ResponseValue::BigInt { value });
                }
            }
            ScalarType::Float => return Ok(ResponseValue::Float { value: v as f64 }),
            ScalarType::Any => {
                return Ok(ResponseValue::U64 { value: v });
            }
            _ => (),
        };

        Ok(self.unexpected_type(Unexpected::Unsigned(v)))
    }

    fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
    where
        E: Error,
    {
        match self.ty {
            ScalarType::Int => {
                if let Ok(value) = i32::try_from(v) {
                    return Ok(ResponseValue::Int { value });
                }
            }
            ScalarType::BigInt | ScalarType::Any => {
                if let Ok(value) = i64::try_from(v) {
                    return Ok(ResponseValue::BigInt { value });
                }
            }
            ScalarType::Float => return Ok(ResponseValue::Float { value: v as f64 }),
            _ => (),
        };

        Ok(self.unexpected_type(Unexpected::Other(&format!("integer {v}"))))
    }

    fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
    where
        E: Error,
    {
        match self.ty {
            ScalarType::Float | ScalarType::Any => Ok(ResponseValue::Float { value: v as f64 }),
            ScalarType::Int if can_coerce_f32_to_int(v) => Ok(ResponseValue::Int { value: v as i32 }),
            ScalarType::BigInt if can_coerce_f32_to_big_int(v) => Ok(ResponseValue::BigInt { value: v as i64 }),
            _ => Ok(self.unexpected_type(Unexpected::Float(v as f64))),
        }
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        match self.ty {
            ScalarType::Float | ScalarType::Any => Ok(ResponseValue::Float { value: v }),
            ScalarType::Int if can_coerce_f64_to_int(v) => Ok(ResponseValue::Int { value: v as i32 }),
            ScalarType::BigInt if can_coerce_f64_to_big_int(v) => Ok(ResponseValue::BigInt { value: v as i64 }),
            _ => Ok(self.unexpected_type(Unexpected::Float(v))),
        }
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        match self.ty {
            ScalarType::String | ScalarType::Any => Ok(ResponseValue::String { value: v.into() }),
            _ => Ok(self.unexpected_type(Unexpected::Str(v))),
        }
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: Error,
    {
        match self.ty {
            ScalarType::String | ScalarType::Any => Ok(ResponseValue::String {
                value: v.into_boxed_str(),
            }),
            _ => Ok(self.unexpected_type(Unexpected::Str(&v))),
        }
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(self.unexpected_type(Unexpected::Bytes(v)))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: Error,
    {
        if self.is_required {
            Ok(self.unexpected_type(Unexpected::Option))
        } else {
            Ok(ResponseValue::Null)
        }
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
        if self.is_required {
            Ok(self.unexpected_type(Unexpected::Unit))
        } else {
            Ok(ResponseValue::Null)
        }
    }

    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        match self.ty {
            ScalarType::Any => {
                let mut list = Vec::new();
                if let Some(size_hist) = seq.size_hint() {
                    list.reserve(size_hist);
                }
                while let Some(value) = seq.next_element_seed(self.clone())? {
                    list.push(value);
                }
                Ok(ResponseValue::List {
                    id: self.ctx.subgraph_response.borrow_mut().data.push_list(list),
                })
            }
            _ => {
                // Try discarding the rest of the list, we might be able to use other parts of
                // the response.
                while seq.next_element::<IgnoredAny>()?.is_some() {}
                Ok(self.unexpected_type(Unexpected::Seq))
            }
        }
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        match self.ty {
            ScalarType::Any => {
                let mut key_values = Vec::new();
                while let Some(key) = map.next_key::<String>()? {
                    let value = map.next_value_seed(self.clone())?;
                    key_values.push((key, value));
                }
                Ok(ResponseValue::Map {
                    id: self.ctx.subgraph_response.borrow_mut().data.push_map(key_values),
                })
            }
            _ => {
                // Try discarding the rest of the map, we might be able to use other parts of
                // the response.
                while map.next_entry::<IgnoredAny, IgnoredAny>()?.is_some() {}
                Ok(self.unexpected_type(Unexpected::Map))
            }
        }
    }

    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::EnumAccess<'de>,
    {
        let _ = data.variant::<IgnoredAny>()?;
        Err(Error::invalid_type(Unexpected::Enum, &self))
    }
}

fn can_coerce_f32_to_int(float: f32) -> bool {
    float.floor() == float && float < (i32::MAX as f32)
}

fn can_coerce_f32_to_big_int(float: f32) -> bool {
    float.floor() == float && float < (i64::MAX as f32)
}

fn can_coerce_f64_to_int(float: f64) -> bool {
    float.floor() == float && float < (i32::MAX as f64)
}

fn can_coerce_f64_to_big_int(float: f64) -> bool {
    float.floor() == float && float < (i64::MAX as f64)
}
