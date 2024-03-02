use std::{
    collections::BTreeMap,
    fmt::{self, Display, Formatter},
    hash::{Hash, Hasher},
    ops::{Deref, DerefMut},
};

use engine_v2_common::{BorrowedValue, BorrowedVariables};
use serde::{Deserialize, Deserializer, Serialize};

use crate::{ConstValue, Name};

/// Variables of a query.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(transparent)]
pub struct Variables(BTreeMap<Name, ConstValue>);

impl Display for Variables {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("{")?;
        for (i, (name, value)) in self.0.iter().enumerate() {
            write!(f, "{}{name}: {value}", if i == 0 { "" } else { ", " })?;
        }
        f.write_str("}")
    }
}

impl Hash for Variables {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for (key, value) in &self.0 {
            key.hash(state);
            value.hash(state);
        }
    }
}

impl<'de> Deserialize<'de> for Variables {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
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

impl From<&BorrowedVariables<'_>> for Variables {
    fn from(variables: &BorrowedVariables<'_>) -> Self {
        let converter = ValueConverter(variables);
        match converter.convert(&variables.root) {
            ConstValue::Object(obj) => Self(obj.into_iter().collect()),
            _ => Self::default(),
        }
    }
}

struct ValueConverter<'a>(&'a BorrowedVariables<'a>);

impl ValueConverter<'_> {
    fn convert(&self, value: &BorrowedValue<'_>) -> ConstValue {
        match value {
            BorrowedValue::Null => ConstValue::Null,
            BorrowedValue::Bool(b) => ConstValue::Boolean(*b),
            BorrowedValue::String(s) => ConstValue::String(s.to_string()),
            BorrowedValue::F64(f) => (*f).into(),
            BorrowedValue::I64(i) => (*i).into(),
            BorrowedValue::U64(u) => (*u).into(),
            BorrowedValue::List(range) => ConstValue::List(self.0[*range].iter().map(|v| self.convert(v)).collect()),
            BorrowedValue::Map(range) => ConstValue::Object(
                self.0[*range]
                    .iter()
                    .map(|(k, v)| (Name::new(k), self.convert(v)))
                    .collect(),
            ),
        }
    }
}
