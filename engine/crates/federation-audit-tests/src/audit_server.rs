use serde::de::DeserializeOwned;

/// API client for the graphql-federation-gateway-audit server
///
/// Can provide all the things required for a test.
#[derive(Clone)]
pub struct AuditServer {
    client: reqwest::blocking::Client,
    url: String,
}

impl AuditServer {
    pub fn new_from_env() -> Self {
        AuditServer {
            client: reqwest::blocking::Client::new(),
            url: std::env::var("AUDIT_SERVER_URL").expect("AUDIT_SERVER_URL env var must be set"),
        }
    }

    pub fn test_suites(&self) -> Vec<TestSuite> {
        self.request::<Vec<String>>("/ids")
            .into_iter()
            .map(|id| TestSuite {
                server: self.clone(),
                id,
            })
            .collect()
    }

    fn request<T: DeserializeOwned>(&self, path: &str) -> T {
        self.client
            .get(format!("{}{}", self.url, path))
            .send()
            .unwrap()
            .error_for_status()
            .unwrap()
            .json()
            .unwrap()
    }
}

/// An individual test suite from graphql-federation-gateway-audit
///
/// Each test suite has a set of subgraphs, and a set of tests that can be
/// run against those subgraphs
#[derive(Clone)]
pub struct TestSuite {
    server: AuditServer,
    pub id: String,
}

impl std::fmt::Debug for TestSuite {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TestSuite").field("id", &self.id).finish()
    }
}

impl TestSuite {
    pub fn tests(&self) -> Vec<Test> {
        self.request("/tests")
    }

    pub fn subgraphs(&self) -> Vec<Subgraph> {
        self.request("/subgraphs")
    }

    pub fn supergraph_sdl(&self) -> String {
        self.server
            .client
            .get(format!("{}/{}/supergraph.graphql", self.server.url, self.id))
            .send()
            .unwrap()
            .error_for_status()
            .unwrap()
            .text()
            .unwrap()
    }

    fn request<T: DeserializeOwned>(&self, path: &str) -> T {
        self.server.request(&format!("/{}{}", self.id, path))
    }
}

/// An individual test from graphql-federation-gateway-audit
///
/// These are simple request/response tests that should be run against
/// the supergraph formed from the parent test suites subgraphs
#[derive(serde::Deserialize, Clone, Debug)]
pub struct Test {
    pub query: String,
    pub expected: ExpectedResponse,
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct ExpectedResponse {
    pub data: Option<serde_json::Value>,
    #[serde(default)]
    pub errors: bool,
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct Subgraph {
    pub name: String,
    pub url: String,
    pub sdl: String,
}
