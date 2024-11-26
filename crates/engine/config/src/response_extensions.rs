use std::{borrow::Cow, str::FromStr};

use gateway_config::telemetry::exporters::ResponseExtensionExporterConfig;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ResponseExtensionConfig {
    /// Whether the traceId is exposed in the grafbase response extension. Defaults to true.
    pub include_trace_id: bool,
    /// Whether the query plan is exposed in the grafbase response extension. Defaults to true.
    pub include_query_plan: bool,
    /// Defines under which conditions the grafbase response extension will be added.
    /// Defaults to a simple header rule, the presence of `x-grafbase-telemetry` is enough.
    pub access_control: Vec<AccessControl>,
}

impl Default for ResponseExtensionConfig {
    fn default() -> Self {
        ResponseExtensionExporterConfig::default().into()
    }
}

impl From<ResponseExtensionExporterConfig> for ResponseExtensionConfig {
    fn from(config: ResponseExtensionExporterConfig) -> Self {
        ResponseExtensionConfig {
            include_trace_id: config.trace_id,
            include_query_plan: config.query_plan,
            access_control: config
                .access_control
                .into_iter()
                .map(|ac| match ac {
                    gateway_config::telemetry::exporters::AccessControl::Header(hac) => {
                        AccessControl::Header(HeaderAccessControl {
                            name: http::HeaderName::from_str(hac.name.as_str()).unwrap(),
                            value: hac.value.map(|v| http::HeaderValue::from_str(v.as_str()).unwrap()),
                        })
                    }
                    gateway_config::exporters::AccessControl::Deny => AccessControl::Deny,
                })
                .collect(),
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum AccessControl {
    Header(HeaderAccessControl),
    Deny,
}

#[derive(Debug)]
pub struct HeaderAccessControl {
    /// Name of the header that must be present.
    pub name: http::HeaderName,
    /// Expected value of the header. If not provided any value will be accepted.
    pub value: Option<http::HeaderValue>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct SerdeHeaderAccessControl<'a> {
    name: Cow<'a, str>,
    value: Option<Cow<'a, str>>,
}

impl serde::Serialize for HeaderAccessControl {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::Error;
        SerdeHeaderAccessControl {
            name: self.name.as_str().into(),
            value: self
                .value
                .as_ref()
                .map(|v| v.to_str())
                .transpose()
                .map_err(S::Error::custom)?
                .map(Into::into),
        }
        .serialize(serializer)
    }
}

impl<'a> serde::Deserialize<'a> for HeaderAccessControl {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'a>,
    {
        use serde::de::Error;
        let SerdeHeaderAccessControl::<'a> { name, value } = serde::Deserialize::deserialize(deserializer)?;
        Ok(HeaderAccessControl {
            name: http::HeaderName::from_str(&name).map_err(D::Error::custom)?,
            value: value
                .map(|v| http::HeaderValue::from_str(&v).map_err(D::Error::custom))
                .transpose()?,
        })
    }
}
