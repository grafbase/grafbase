use ascii::AsciiString;
use serde::{
    de::{MapAccess, Visitor},
    Deserialize, Deserializer,
};
use serde_dynamic_string::DynamicString;
use std::{collections::HashMap, fmt::Formatter, str::FromStr};

/// List of headers to be sent on export requests
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Headers(pub(crate) Vec<(AsciiString, DynamicString<AsciiString>)>);

impl Headers {
    /// Consume self and return the inner list
    pub fn into_inner(self) -> Vec<(AsciiString, DynamicString<AsciiString>)> {
        self.0
    }

    /// Gets the headers as a referenced slice
    pub fn inner(&self) -> &[(AsciiString, DynamicString<AsciiString>)] {
        &self.0
    }

    /// Consume self and return a map of header/header_value as ascii strings
    pub fn into_map(self) -> HashMap<String, String> {
        self.into_inner()
            .into_iter()
            .map(|(name, value)| (name.to_string(), value.to_string()))
            .collect()
    }

    pub fn len(&self) -> usize {
        self.inner().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl IntoIterator for Headers {
    type Item = (AsciiString, DynamicString<AsciiString>);
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl From<Vec<(AsciiString, AsciiString)>> for Headers {
    fn from(headers: Vec<(AsciiString, AsciiString)>) -> Self {
        Self(headers.into_iter().map(|(k, v)| (k, DynamicString::from(v))).collect())
    }
}

impl<'de> Deserialize<'de> for Headers {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(HeaderMapVisitor)
    }
}

struct HeaderMapVisitor;
impl<'de> Visitor<'de> for HeaderMapVisitor {
    type Value = Headers;

    fn expecting(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "a key-value map")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut headers = Vec::with_capacity(map.size_hint().unwrap_or(0));

        while let Some((key, value)) = map.next_entry::<String, String>()? {
            let header_name = AsciiString::from_ascii(key).map_err(|err| serde::de::Error::custom(err.to_string()))?;

            let header_value =
                DynamicString::from_str(&value).map_err(|err| serde::de::Error::custom(err.to_string()))?;

            headers.push((header_name, header_value));
        }

        Ok(Headers(headers))
    }
}
