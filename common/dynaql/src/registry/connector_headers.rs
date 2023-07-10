use crate::RequestHeaders;

#[derive(Clone, Default, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
/// Headers we should send to a connectors downstream server
pub struct ConnectorHeaders(Vec<(String, ConnectorHeaderValue)>);

impl ConnectorHeaders {
    pub fn new(headers: impl IntoIterator<Item = (String, ConnectorHeaderValue)>) -> Self {
        ConnectorHeaders(headers.into_iter().collect())
    }
}

impl ConnectorHeaders {
    pub fn build_header_vec<'a>(
        &'a self,
        request_headers: &'a RequestHeaders,
    ) -> Vec<(&'a str, &'a str)> {
        self.0
            .iter()
            .filter_map(|(name, value)| match value {
                ConnectorHeaderValue::Static(static_value) => {
                    Some((name.as_str(), static_value.as_str()))
                }
                ConnectorHeaderValue::Forward(header_name) => {
                    Some((name.as_str(), request_headers.find(header_name)?))
                }
            })
            .collect()
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum ConnectorHeaderValue {
    /// We should send a static value for this header
    Static(String),
    /// We should pull the value for this header from the named header in the incoming
    /// request
    Forward(String),
}
