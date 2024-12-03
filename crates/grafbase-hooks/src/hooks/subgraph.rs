use crate::wit::HttpMethod;

/// A structure representing a request that will be sent to a subgraph.
pub struct SubgraphRequest {
    subgraph_name: String,
    method: HttpMethod,
    url: String,
}

impl SubgraphRequest {
    pub(crate) fn new(subgraph_name: String, method: HttpMethod, url: String) -> Self {
        Self {
            subgraph_name,
            method,
            url,
        }
    }

    /// Returns the name of the subgraph.
    pub fn subgraph_name(&self) -> &str {
        &self.subgraph_name
    }

    /// Returns the URl for this request.
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Returns the HTTP method for this request.
    pub fn method(&self) -> HttpMethod {
        self.method
    }
}
