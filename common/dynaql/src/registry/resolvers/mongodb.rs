#[serde_with::minify_field_names(serialize = "minified", deserialize = "minified")]
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, Hash, PartialEq, Eq)]
pub struct HttpResolver {
    pub method: String,
    pub url: String,
    pub api_name: String,
    pub path_parameters: Vec<PathParameter>,
    pub query_parameters: Vec<QueryParameter>,
    pub request_body: Option<RequestBody>,
    pub expected_status: ExpectedStatusCode,
}
