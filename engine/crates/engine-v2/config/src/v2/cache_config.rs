use std::collections::BTreeMap;
use std::fmt::Formatter;
use std::str::FromStr;
use std::time::Duration;

use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use federated_graph::{FieldId, ObjectId};

#[derive(Default, Debug, Hash, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct CacheConfigs {
    pub rules: BTreeMap<CacheConfigTarget, CacheConfig>,
}

impl CacheConfigs {
    pub fn rule(&self, key: CacheConfigTarget) -> Option<&CacheConfig> {
        self.rules.get(&key)
    }
}

#[derive(Default, Debug, Hash, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct CacheConfig {
    pub max_age: Duration,
    pub stale_while_revalidate: Duration,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub enum CacheConfigTarget {
    Object(ObjectId),
    Field(FieldId),
}

#[derive(Debug, thiserror::Error)]
pub enum CacheConfigError {
    #[error("Parsing error: {0}")]
    Parse(String),
}

impl std::fmt::Display for CacheConfigTarget {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CacheConfigTarget::Object(object_id) => write!(f, "o{}", object_id.0),
            CacheConfigTarget::Field(field_id) => write!(f, "f{}", field_id.0),
        }
    }
}

impl FromStr for CacheConfigTarget {
    type Err = CacheConfigError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() < 2 {
            return Err(CacheConfigError::Parse("empty cache config target".to_string()));
        }

        let s = s.as_bytes();

        let first = char::from(s[0]);
        let id = std::str::from_utf8(&s[1..])
            .map_err(|e| CacheConfigError::Parse(e.to_string()))?
            .parse::<usize>()
            .map_err(|e| CacheConfigError::Parse(e.to_string()))?;

        match first {
            'o' => Ok(CacheConfigTarget::Object(ObjectId(id))),
            'f' => Ok(CacheConfigTarget::Field(FieldId(id))),
            _ => Err(CacheConfigError::Parse("invalid id format".to_string())),
        }
    }
}

impl Serialize for CacheConfigTarget {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let repr = self.to_string();
        serializer.serialize_str(&repr)
    }
}

impl<'de> Deserialize<'de> for CacheConfigTarget {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let str = String::deserialize(deserializer)?;

        CacheConfigTarget::from_str(&str).map_err(|e| Error::custom(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use federated_graph::{FieldId, ObjectId};

    use crate::v2::CacheConfigTarget;

    #[test]
    fn test_custom_serde() {
        let field = CacheConfigTarget::Field(FieldId(0));
        let object = CacheConfigTarget::Object(ObjectId(0));

        let field_json_repr = serde_json::to_string(&field).unwrap();
        let object_json_repr = serde_json::to_string(&object).unwrap();

        assert_eq!("\"f0\"", field_json_repr);
        assert_eq!("\"o0\"", object_json_repr);

        let field_enum: CacheConfigTarget = serde_json::from_str(&field_json_repr).unwrap();
        let object_enum: CacheConfigTarget = serde_json::from_str(&object_json_repr).unwrap();

        assert_eq!(field, field_enum);
        assert_eq!(object, object_enum);
    }
}
