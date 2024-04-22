use std::collections::BTreeMap;

use super::variable_resolve_definition::VariableResolveDefinition;

#[serde_with::minify_field_names(serialize = "minified", deserialize = "minified")]
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct HttpResolver {
    pub method: String,
    pub url: String,
    pub api_name: String,
    pub path_parameters: Vec<PathParameter>,
    pub query_parameters: Vec<QueryParameter>,
    pub request_body: Option<RequestBody>,
    pub expected_status: ExpectedStatusCode,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct PathParameter {
    pub name: String,
    pub variable_resolve_definition: VariableResolveDefinition,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct QueryParameter {
    pub name: String,
    pub variable_resolve_definition: VariableResolveDefinition,
    pub encoding_style: QueryParameterEncodingStyle,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct RequestBody {
    pub variable_resolve_definition: VariableResolveDefinition,
    pub content_type: RequestBodyContentType,
}

#[derive(Clone, Copy, Debug, serde::Deserialize, serde::Serialize, PartialEq)]
pub enum QueryParameterEncodingStyle {
    Form,
    FormExploded,
    DeepObject,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub enum RequestBodyContentType {
    Json,
    FormEncoded(BTreeMap<String, QueryParameterEncodingStyle>),
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, PartialEq)]
pub enum ExpectedStatusCode {
    Exact(u16),
    Range(std::ops::Range<u16>),
}

impl ExpectedStatusCode {
    pub fn is_success(&self) -> bool {
        match self {
            ExpectedStatusCode::Exact(code) => 200 <= *code && *code < 300,
            ExpectedStatusCode::Range(code_range) => code_range.contains(&200) && code_range.end < 300,
        }
    }
}
