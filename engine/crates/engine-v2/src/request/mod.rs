pub(crate) mod extensions;

use engine_value::{ConstValue, Name};
use extensions::RequestExtensions;
use serde::Deserializer;
use std::{
    collections::BTreeMap,
    fmt,
    hash::Hasher,
    ops::{Deref, DerefMut},
};

#[derive(serde::Deserialize)]
#[serde(untagged)]
pub(crate) enum BatchRequest {
    Single(Request),
    Batch(Vec<Request>),
}

#[derive(serde::Deserialize, Debug)]
pub(crate) struct Request {
    #[serde(default)]
    pub query: Option<String>,
    #[serde(default, rename = "operationName")]
    pub operation_name: Option<String>,
    #[serde(default)]
    pub doc_id: Option<String>,
    #[serde(default)]
    pub variables: Variables,
    #[serde(default)]
    pub extensions: RequestExtensions,
}

/// Variables of a query.
#[derive(Debug, Clone, Default, serde::Serialize)]
#[serde(transparent)]
pub struct Variables(BTreeMap<Name, ConstValue>);

impl fmt::Display for Variables {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("{")?;

        for (i, (name, value)) in self.0.iter().enumerate() {
            write!(f, "{}{name}: {value}", if i == 0 { "" } else { ", " })?;
        }

        f.write_str("}")
    }
}

impl std::hash::Hash for Variables {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for (key, value) in &self.0 {
            key.hash(state);
            value.hash(state);
        }
    }
}

impl<'de> serde::Deserialize<'de> for Variables {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Self(
            <Option<BTreeMap<Name, ConstValue>>>::deserialize(deserializer)?.unwrap_or_default(),
        ))
    }
}

impl Variables {
    /// Get the variables from a GraphQL value.
    ///
    /// If the value is not a map, then no variables will be returned.
    #[must_use]
    pub fn from_value(value: ConstValue) -> Self {
        match value {
            ConstValue::Object(obj) => Self(obj.into_iter().collect()),
            _ => Self::default(),
        }
    }

    /// Get the values from a JSON value.
    ///
    /// If the value is not a map or the keys of a map are not valid GraphQL names, then no
    /// variables will be returned.
    #[must_use]
    pub fn from_json(value: serde_json::Value) -> Self {
        ConstValue::from_json(value).map(Self::from_value).unwrap_or_default()
    }

    /// Get the variables as a GraphQL value.
    #[must_use]
    pub fn into_value(self) -> ConstValue {
        ConstValue::Object(self.0.into_iter().collect())
    }
}

impl IntoIterator for Variables {
    type Item = (Name, ConstValue);
    type IntoIter = <BTreeMap<Name, ConstValue> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl From<Variables> for ConstValue {
    fn from(variables: Variables) -> Self {
        variables.into_value()
    }
}

impl Deref for Variables {
    type Target = BTreeMap<Name, ConstValue>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Variables {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub(crate) struct QueryParamsRequest(Request);

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
                .map(serde_json::from_str)
                .transpose()
                .map_err(serde::de::Error::custom)?
                .unwrap_or_default(),
            extensions: extensions
                .as_deref()
                .map(serde_json::from_str)
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
