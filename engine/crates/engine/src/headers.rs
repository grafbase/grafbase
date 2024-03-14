use std::collections::HashMap;

/// The headers that were provided in the HTTP request to engine.
///
/// Certain connectors use these to forward headers on, depending on their configuration.
#[derive(Default, Clone)]
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
                .map(|(name, value)| {
                    let mut name = name.into();
                    name.make_ascii_lowercase();
                    (name, value.into())
                })
                .collect(),
        )
    }

    pub fn find<'a>(&'a self, expected_name: &str) -> Option<&'a str> {
        let expected_name = expected_name.to_ascii_lowercase();
        self.0
            .iter()
            .find(|(name, _)| name.as_str() == expected_name)
            .map(|(_, value)| value.as_str())
    }
}

impl From<&HashMap<String, String>> for RequestHeaders {
    fn from(value: &HashMap<String, String>) -> Self {
        RequestHeaders::new(value.clone())
    }
}

impl From<HashMap<String, String>> for RequestHeaders {
    fn from(value: HashMap<String, String>) -> Self {
        RequestHeaders::new(value)
    }
}

impl<N, V> std::iter::FromIterator<(N, V)> for RequestHeaders
where
    N: Into<String>,
    V: Into<String>,
{
    fn from_iter<T: IntoIterator<Item = (N, V)>>(iter: T) -> Self {
        RequestHeaders::new(iter)
    }
}
