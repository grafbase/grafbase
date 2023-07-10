use std::collections::HashMap;

/// The headers that were provided in the HTTP request to dynaql.
///
/// Certain connectors use these to forward headers on, depending on their configuration.
#[derive(Default)]
pub struct RequestHeaders(Vec<(String, String)>);

impl RequestHeaders {
    pub fn new<N, V>(headers: impl IntoIterator<Item = (N, V)>) -> Self
    where
        N: Into<String>,
        V: Into<String>,
    {
        RequestHeaders(
            headers
                .into_iter()
                .map(|(n, v)| (n.into(), v.into()))
                .collect(),
        )
    }

    pub fn find(&self, expected_name: &str) -> Option<&str> {
        self.0
            .iter()
            .find(|(name, _)| name == expected_name)
            .map(|(_, value)| value.as_str())
    }
}

impl From<&HashMap<String, String>> for RequestHeaders {
    fn from(value: &HashMap<String, String>) -> Self {
        RequestHeaders::new(value.clone())
    }
}
