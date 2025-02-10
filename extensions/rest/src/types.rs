#[derive(Debug)]
pub struct RestEndpoint {
    pub subgraph_name: String,
    pub args: RestEndpointArgs,
}

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RestEndpointArgs {
    pub name: String,
    pub http: HttpSettings,
}

#[derive(serde::Deserialize, Debug)]
pub struct HttpSettings {
    #[serde(rename = "baseURL")]
    pub base_url: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rest<'a> {
    pub endpoint: &'a str,
    pub http: HttpCall<'a>,
    pub selection: &'a str,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpCall<'a> {
    pub method: HttpMethod,
    pub path: &'a str,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    Get,
}

impl From<HttpMethod> for ::http::Method {
    fn from(method: HttpMethod) -> Self {
        match method {
            HttpMethod::Get => Self::GET,
        }
    }
}
