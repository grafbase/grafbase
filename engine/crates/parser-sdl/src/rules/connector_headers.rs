use registry_v2::ConnectorHeaderValue;

#[derive(Debug, serde::Deserialize)]
pub struct IntrospectionHeader {
    pub name: String,
    pub value: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(try_from = "HeaderDeserialize")]
pub struct Header {
    pub name: String,

    pub value: ConnectorHeaderValue,
}

impl TryFrom<HeaderDeserialize> for Header {
    type Error = &'static str;

    fn try_from(header: HeaderDeserialize) -> Result<Self, Self::Error> {
        let value = match (header.value, header.forward) {
            (None, Some(header_name)) => ConnectorHeaderValue::Forward(header_name),
            (Some(value), None) => ConnectorHeaderValue::Static(value),
            (None, None) => return Err("a header must have one of value or forward"),
            (Some(_), Some(_)) => return Err("a header can't have both value and forward"),
        };

        Ok(Header {
            name: header.name,
            value,
        })
    }
}

#[derive(Debug, serde::Deserialize)]
struct HeaderDeserialize {
    name: String,

    /// A hardcoded value for the header
    value: Option<String>,

    /// We should forward this header from the named header in the incoming request.
    forward: Option<String>,
}
