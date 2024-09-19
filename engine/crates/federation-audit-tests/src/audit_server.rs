use serde::de::DeserializeOwned;

/// API client for the graphql-federation-gateway-audit server
///
/// Can provide all the things required for a test.
#[derive(Clone)]
pub struct AuditServer {
    client: reqwest::Client,
    url: String,
}

impl AuditServer {
    pub async fn test_suites(&self) -> Vec<TestSuite> {
        self.request::<Vec<String>>("/ids")
            .await
            .into_iter()
            .map(|id| TestSuite {
                server: self.clone(),
                id,
            })
            .collect()
    }

    async fn request<T: DeserializeOwned>(&self, path: &str) -> T {
        self.client
            .get(format!("{}{}", self.url, path))
            .send()
            .await
            .unwrap()
            .error_for_status()
            .unwrap()
            .json()
            .await
            .unwrap()
    }
}

/// An individual test suite from graphql-federation-gateway-audit
///
/// Each test suite has a set of subgraphs, and a set of tests that can be
/// run against those subgraphs
pub struct TestSuite {
    server: AuditServer,
    pub id: String,
}

impl TestSuite {
    pub async fn tests(&self) -> Vec<Test> {
        self.request("/tests").await
    }

    pub async fn subgraphs(&self) -> Vec<Subgraph> {
        self.request("/subgraphs").await
    }

    pub async fn supergraph_sdl(&self) -> String {
        self.server
            .client
            .get(format!("{}/{}/supergraph.graphql", self.server.url, self.id))
            .send()
            .await
            .unwrap()
            .error_for_status()
            .unwrap()
            .text()
            .await
            .unwrap()
    }

    async fn request<T: DeserializeOwned>(&self, path: &str) -> T {
        self.server.request(&format!("/{}{}", self.id, path)).await
    }
}

/// An individual test from graphql-federation-gateway-audit
///
/// These are simple request/response tests that should be run against
/// the supergraph formed from the parent test suites subgraphs
#[derive(serde::Deserialize)]
pub struct Test {
    pub query: String,
    pub expected: ExpectedResponse,
}

#[derive(serde::Deserialize)]
pub struct ExpectedResponse {
    pub data: Option<serde_json::Value>,
    pub errors: bool,
}

#[derive(serde::Deserialize)]
pub struct Subgraph {
    pub name: String,
    pub url: String,
    pub sdl: String,
}
