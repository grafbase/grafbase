use crate::wit;

/// HTTP headers. Will be the gateway headers for authentication/authorization extensions and
/// the subgraph headers for resolvers.
pub struct Headers(wit::Headers);

impl From<wit::Headers> for Headers {
    fn from(headers: wit::Headers) -> Self {
        Self(headers)
    }
}

impl Headers {
    /// Get the header value for the specified header name.
    pub fn get(&self, name: &str) -> Option<String> {
        self.0.get(name)
    }

    /// Get all header entries.
    pub fn entries(&self) -> Vec<(String, String)> {
        self.0.entries()
    }
}
