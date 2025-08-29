use schema::ScalarType;
use serde::de::{DeserializeSeed, Error, IgnoredAny, Unexpected, Visitor};
use walker::Walk;

use crate::{
    prepare::FieldShapeRecord,
    response::{GraphqlError, ResponseValue},
};

use super::SeedState;

#[derive(Clone, Copy)]
pub(crate) struct ScalarTypeSeed<'ctx, 'parent, 'state> {
    pub state: &'state SeedState<'ctx, 'parent>,
    pub field: &'ctx FieldShapeRecord,
    pub is_required: bool,
    pub ty: ScalarType,
}

impl<'de> DeserializeSeed<'de> for ScalarTypeSeed<'_, '_, '_> {
    type Value = ResponseValue;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }
}

impl ScalarTypeSeed<'_, '_, '_> {
    fn unexpected_type(&self, value: Unexpected<'_>) -> <Self as Visitor<'_>>::Value {
        let expected = match &self.ty {
            ScalarType::String => "a String value",
            ScalarType::Float => "a Float value",
            ScalarType::Int => "an Int value",
            ScalarType::Unknown => "a JSON value",
            ScalarType::Boolean => "a Boolean value",
        };
        tracing::error!(
            "invalid type: {}, expected {} at '{}'",
            value,
            expected,
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

impl<'de> Visitor<'de> for ScalarTypeSeed<'_, '_, '_> {
    type Value = ResponseValue;
    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("any value?")
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(match self.ty {
            ScalarType::Boolean | ScalarType::Unknown => v.into(),
            _ => self.unexpected_type(Unexpected::Bool(v)),
        })
    }

    fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(match self.ty {
            ScalarType::Int | ScalarType::Unknown => ResponseValue::Int { value: v as i32 },
            ScalarType::Float => ResponseValue::Float { value: v as f64 },
            _ => self.unexpected_type(Unexpected::Signed(v.into())),
        })
    }

    fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(match self.ty {
            ScalarType::Int | ScalarType::Unknown => ResponseValue::Int { value: v as i32 },
            ScalarType::Float => ResponseValue::Float { value: v as f64 },
            _ => self.unexpected_type(Unexpected::Signed(v.into())),
        })
    }

    fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(match self.ty {
            ScalarType::Int | ScalarType::Unknown => ResponseValue::Int { value: v },
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
            ScalarType::Unknown => ResponseValue::I64 { value: v },
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
            ScalarType::Unknown => {
                if let Ok(value) = i64::try_from(v) {
                    return Ok(ResponseValue::I64 { value });
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
            ScalarType::Int | ScalarType::Unknown => ResponseValue::Int { value: v as i32 },
            ScalarType::Float => ResponseValue::Float { value: v as f64 },
            _ => self.unexpected_type(Unexpected::Unsigned(v.into())),
        })
    }

    fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(match self.ty {
            ScalarType::Int | ScalarType::Unknown => ResponseValue::Int { value: v as i32 },
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
            ScalarType::Unknown => ResponseValue::I64 { value: v as i64 },
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
            ScalarType::Float => return Ok(ResponseValue::Float { value: v as f64 }),
            ScalarType::Unknown => {
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
            ScalarType::Unknown => {
                if let Ok(value) = i64::try_from(v) {
                    return Ok(ResponseValue::I64 { value });
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
            ScalarType::Float | ScalarType::Unknown => Ok(ResponseValue::Float { value: v as f64 }),
            ScalarType::Int if can_coerce_f32_to_int(v) => Ok(ResponseValue::Int { value: v as i32 }),
            _ => Ok(self.unexpected_type(Unexpected::Float(v as f64))),
        }
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        match self.ty {
            ScalarType::Float | ScalarType::Unknown => Ok(ResponseValue::Float { value: v }),
            ScalarType::Int if can_coerce_f64_to_int(v) => Ok(ResponseValue::Int { value: v as i32 }),
            _ => Ok(self.unexpected_type(Unexpected::Float(v))),
        }
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        match self.ty {
            ScalarType::String | ScalarType::Unknown => Ok(ResponseValue::String { value: v.into() }),
            _ => Ok(self.unexpected_type(Unexpected::Str(v))),
        }
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: Error,
    {
        match self.ty {
            ScalarType::String | ScalarType::Unknown => Ok(ResponseValue::String { value: v }),
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
            ScalarType::Unknown => {
                let (list_id, mut list) = self.state.response.borrow_mut().data.take_next_list();
                let offset = list.len();
                if let Some(size_hist) = seq.size_hint() {
                    list.reserve(size_hist);
                }
                let result = ingest_seq(
                    &mut seq,
                    ScalarTypeSeed {
                        state: self.state,
                        field: self.field,
                        is_required: false,
                        ty: ScalarType::Unknown,
                    },
                    &mut list,
                );

                let limit = list.len() - offset;
                self.state.response.borrow_mut().data.restore_list(list_id, list);

                result?;

                Ok(ResponseValue::List {
                    id: list_id,
                    offset: offset as u32,
                    limit: limit as u32,
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
            ScalarType::Unknown => {
                let (map_id, mut key_values) = self.state.response.borrow_mut().data.take_next_map();
                let offset = key_values.len();
                let result = ingest_map(
                    &mut map,
                    ScalarTypeSeed {
                        state: self.state,
                        field: self.field,
                        is_required: false,
                        ty: ScalarType::Unknown,
                    },
                    &mut key_values,
                );
                let limit = key_values.len() - offset;
                self.state.response.borrow_mut().data.restore_map(map_id, key_values);

                result?;

                Ok(ResponseValue::Map {
                    id: map_id,
                    offset: offset as u32,
                    limit: limit as u32,
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

fn ingest_seq<'de, A>(
    mut seq: A,
    seed: ScalarTypeSeed<'_, '_, '_>,
    list: &mut Vec<ResponseValue>,
) -> Result<(), A::Error>
where
    A: serde::de::SeqAccess<'de>,
{
    while let Some(value) = seq.next_element_seed(seed)? {
        list.push(value);
    }
    Ok(())
}

fn ingest_map<'de, A>(
    mut map: A,
    seed: ScalarTypeSeed<'_, '_, '_>,
    key_values: &mut Vec<(String, ResponseValue)>,
) -> Result<(), A::Error>
where
    A: serde::de::MapAccess<'de>,
{
    while let Some(key) = map.next_key::<String>()? {
        let value = map.next_value_seed(seed)?;
        key_values.push((key, value));
    }
    Ok(())
}

fn can_coerce_f32_to_int(float: f32) -> bool {
    float.floor() == float && float < (i32::MAX as f32)
}

fn can_coerce_f64_to_int(float: f64) -> bool {
    float.floor() == float && float < (i32::MAX as f64)
}
