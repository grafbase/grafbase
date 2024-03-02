use serde::{
    de::{DeserializeSeed, Visitor},
    Deserialize, Deserializer,
};
use std::borrow::Cow;

use crate::{BorrowedValue, BorrowedVariables};

pub(super) fn deserialize_non_empty_vec<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    use serde::de::Error as _;

    let v = <Vec<T>>::deserialize(deserializer)?;
    if v.is_empty() {
        Err(D::Error::invalid_length(0, &"a non-empty sequence"))
    } else {
        Ok(v)
    }
}

impl<'de: 'a, 'a> Deserialize<'de> for BorrowedVariables<'a> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut seed = BorrowedVariablesSeed::default();
        let root = deserializer.deserialize_any(&mut seed)?;
        if !matches!(root, BorrowedValue::Null | BorrowedValue::Map(_)) {
            return Err(serde::de::Error::custom("variables must be a map or null"));
        }
        Ok(BorrowedVariables {
            root,
            values: seed.values,
            key_values: seed.key_values,
        })
    }
}

#[derive(Default)]
struct BorrowedVariablesSeed<'a> {
    values: Vec<BorrowedValue<'a>>,
    values_buffer_pool: Vec<Vec<BorrowedValue<'a>>>,
    key_values: Vec<(Cow<'a, str>, BorrowedValue<'a>)>,
    key_values_buffer_pool: Vec<Vec<(Cow<'a, str>, BorrowedValue<'a>)>>,
}

impl<'de> DeserializeSeed<'de> for &mut BorrowedVariablesSeed<'de> {
    type Value = BorrowedValue<'de>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_option(self)
    }
}

impl<'de> Visitor<'de> for &mut BorrowedVariablesSeed<'de> {
    type Value = BorrowedValue<'de>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a JSON value")
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(BorrowedValue::Null)
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut buffer = self.key_values_buffer_pool.pop().unwrap_or_default();

        while let Some(key) = map.next_key::<Cow<'de, str>>()? {
            let value: BorrowedValue<'de> = map.next_value_seed(&mut *self)?;
            buffer.push((key, value));
        }

        let start = self.key_values.len();
        buffer.sort_unstable_by(|a, b| a.0.cmp(&b.0));
        self.key_values.append(&mut buffer);
        self.key_values_buffer_pool.push(buffer);
        let end = self.key_values.len();

        Ok(BorrowedValue::Map((start..end).into()))
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut buffer = self.values_buffer_pool.pop().unwrap_or_default();

        while let Some(value) = seq.next_element_seed(&mut *self)? {
            buffer.push(value);
        }

        let start = self.values.len();
        self.values.append(&mut buffer);
        self.values_buffer_pool.push(buffer);
        let end = self.values.len();

        Ok(BorrowedValue::List((start..end).into()))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(BorrowedValue::String(v.to_string().into()))
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(BorrowedValue::String(v.into()))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(BorrowedValue::String(v.into()))
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(BorrowedValue::Bool(v))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(BorrowedValue::I64(v))
    }

    fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(BorrowedValue::I64(v as i64))
    }

    fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(BorrowedValue::I64(v as i64))
    }

    fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(BorrowedValue::I64(v as i64))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(BorrowedValue::U64(v))
    }

    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(BorrowedValue::U64(v as u64))
    }

    fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(BorrowedValue::U64(v as u64))
    }

    fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(BorrowedValue::U64(v as u64))
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(BorrowedValue::F64(v))
    }

    fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(BorrowedValue::F64(v as f64))
    }
}
