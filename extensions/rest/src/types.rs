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
    body: Option<Body>,
}

impl Rest<'_> {
    pub fn body(&self) -> Option<&serde_json::Value> {
        self.body.as_ref().and_then(|body| {
            body.r#static
                .as_ref()
                .or_else(|| body.selection.as_ref().and_then(|s| s.input.as_ref()))
        })
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Body {
    pub selection: Option<RestInput>,
    pub r#static: Option<serde_json::Value>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestInput {
    input: Option<serde_json::Value>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpCall<'a> {
    pub method: HttpMethod,
    pub path: &'a str,
}

#[derive(Debug, Clone, Copy, serde::Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    Get,
    Post,
}

impl From<HttpMethod> for ::http::Method {
    fn from(method: HttpMethod) -> Self {
        match method {
            HttpMethod::Get => Self::GET,
            HttpMethod::Post => Self::POST,
        }
    }
}
