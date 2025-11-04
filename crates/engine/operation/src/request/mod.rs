pub mod extensions;

use extensions::RequestExtensions;
use serde::de::value::{EnumAccessDeserializer, MapAccessDeserializer, SeqAccessDeserializer, UnitDeserializer};
use serde::{
    Deserialize as _, Deserializer,
    de::{self, IntoDeserializer, MapAccess, SeqAccess, Visitor},
};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    fmt,
    hash::Hasher,
    ops::{Deref, DerefMut},
};

pub enum BatchRequest {
    Single(Request),
    Batch(Vec<Request>),
}

impl<'de> serde::Deserialize<'de> for BatchRequest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct BatchRequestVisitor;

        impl BatchRequestVisitor {
            fn delegate<'de, T, E>(value: T) -> Result<BatchRequest, E>
            where
                T: IntoDeserializer<'de, E>,
                E: de::Error,
            {
                Request::deserialize(value.into_deserializer()).map(BatchRequest::Single)
            }
        }

        impl<'de> Visitor<'de> for BatchRequestVisitor {
            type Value = BatchRequest;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a GraphQL request or batch of requests")
            }

            fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let requests = Vec::<Request>::deserialize(SeqAccessDeserializer::new(seq))?;
                Ok(BatchRequest::Batch(requests))
            }

            fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let request = Request::deserialize(MapAccessDeserializer::new(map))?;
                Ok(BatchRequest::Single(request))
            }

            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
            where
                A: de::EnumAccess<'de>,
            {
                let request = Request::deserialize(EnumAccessDeserializer::new(data))?;
                Ok(BatchRequest::Single(request))
            }

            fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: Deserializer<'de>,
            {
                Request::deserialize(deserializer).map(BatchRequest::Single)
            }

            fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: Deserializer<'de>,
            {
                Request::deserialize(deserializer).map(BatchRequest::Single)
            }

            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Request::deserialize(serde_json::Value::Null)
                    .map_err(E::custom)
                    .map(BatchRequest::Single)
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Request::deserialize(UnitDeserializer::new()).map(BatchRequest::Single)
            }

            fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Self::delegate(value)
            }

            fn visit_i8<E>(self, value: i8) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Self::delegate(value)
            }

            fn visit_i16<E>(self, value: i16) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Self::delegate(value)
            }

            fn visit_i32<E>(self, value: i32) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Self::delegate(value)
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Self::delegate(value)
            }

            fn visit_u8<E>(self, value: u8) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Self::delegate(value)
            }

            fn visit_u16<E>(self, value: u16) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Self::delegate(value)
            }

            fn visit_u32<E>(self, value: u32) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Self::delegate(value)
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Self::delegate(value)
            }

            fn visit_f32<E>(self, value: f32) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Self::delegate(value)
            }

            fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Self::delegate(value)
            }

            fn visit_char<E>(self, value: char) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Self::delegate(value)
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Self::delegate(value)
            }

            fn visit_borrowed_str<E>(self, value: &'de str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Self::delegate(value)
            }

            fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Self::delegate(value)
            }

            fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Self::delegate(value)
            }

            fn visit_borrowed_bytes<E>(self, value: &'de [u8]) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Self::delegate(value)
            }

            fn visit_byte_buf<E>(self, value: Vec<u8>) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Self::delegate(value)
            }
        }

        deserializer.deserialize_any(BatchRequestVisitor)
    }
}

#[derive(serde::Deserialize, Debug)]
pub struct Request {
    #[serde(default)]
    pub query: Option<String>,
    #[serde(default, rename = "operationName")]
    pub operation_name: Option<String>,
    #[serde(default)]
    pub doc_id: Option<String>,
    #[serde(default)]
    pub variables: RawVariables,
    #[serde(default)]
    pub extensions: RequestExtensions,
}

/// Variables of a query.
#[derive(Debug, Clone, Default, serde::Serialize)]
#[serde(transparent)]
pub struct RawVariables(BTreeMap<String, Value>);

impl fmt::Display for RawVariables {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("{")?;

        for (i, (name, value)) in self.0.iter().enumerate() {
            write!(f, "{}{name}: {value}", if i == 0 { "" } else { ", " })?;
        }

        f.write_str("}")
    }
}

impl std::hash::Hash for RawVariables {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for (key, value) in &self.0 {
            key.hash(state);
            value.hash(state);
        }
    }
}

impl<'de> serde::Deserialize<'de> for RawVariables {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Self(
            <Option<BTreeMap<String, Value>>>::deserialize(deserializer)?.unwrap_or_default(),
        ))
    }
}

impl RawVariables {
    /// Get the variables from a GraphQL value.
    ///
    /// If the value is not a map, then no variables will be returned.
    #[must_use]
    pub fn from_value(value: Value) -> Self {
        match value {
            Value::Object(obj) => Self(obj.into_iter().collect()),
            _ => Self::default(),
        }
    }

    /// Get the variables as a GraphQL value.
    #[must_use]
    pub fn into_value(self) -> Value {
        Value::Object(self.0.into_iter().collect())
    }
}

impl IntoIterator for RawVariables {
    type Item = (String, Value);
    type IntoIter = <BTreeMap<String, Value> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl From<RawVariables> for Value {
    fn from(variables: RawVariables) -> Self {
        variables.into_value()
    }
}

impl Deref for RawVariables {
    type Target = BTreeMap<String, Value>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RawVariables {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub struct QueryParamsRequest(Request);

impl From<QueryParamsRequest> for Request {
    fn from(QueryParamsRequest(request): QueryParamsRequest) -> Request {
        request
    }
}

impl<'de> serde::Deserialize<'de> for QueryParamsRequest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let QueryParams {
            query,
            doc_id,
            variables,
            operation_name,
            extensions,
        } = QueryParams::deserialize(deserializer)?;

        Ok(QueryParamsRequest(Request {
            query,
            operation_name,
            doc_id,
            variables: variables
                .as_deref()
                .map(sonic_rs::from_str)
                .transpose()
                .map_err(serde::de::Error::custom)?
                .unwrap_or_default(),
            extensions: extensions
                .as_deref()
                .map(sonic_rs::from_str)
                .transpose()
                .map_err(serde::de::Error::custom)?
                .unwrap_or_default(),
        }))
    }
}

#[derive(serde::Deserialize)]
struct QueryParams {
    #[serde(default)]
    query: Option<String>,
    #[serde(default)]
    doc_id: Option<String>,
    #[serde(default)]
    variables: Option<String>,
    #[serde(default)]
    operation_name: Option<String>,
    #[serde(default)]
    extensions: Option<String>,
}
