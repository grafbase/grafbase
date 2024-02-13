use serde::Deserializer;

use crate::Request;

/// GET request have a specific format, values are encoded in JSON, but the root fields are url
/// encoded. To ensure that the HTTP framework doesn't mess with it, we're first trying to
/// deserialize all values as strings.
///
/// ```sh
/// curl --get http://localhost:4000/graphql \
///   --header 'content-type: application/json' \
///   --data-urlencode 'query={__typename}' \
///   --data-urlencode 'extensions={"persistedQuery":{"version":1,"sha256Hash":"ecf4edb46db40b5132295c0291d62fb65d6759a9eedfa4d5d612dd5ec54a6b38"}}'
/// ```
pub struct QueryParamRequest {
    request: Request,
}

impl From<QueryParamRequest> for Request {
    fn from(QueryParamRequest { request }: QueryParamRequest) -> Request {
        request
    }
}

impl<'de> serde::Deserialize<'de> for QueryParamRequest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let query_params = QueryParams::deserialize(deserializer)?;
        // Avoid unecessary serde round trips by creating a JSON string manually.
        // This also ensures we stay as close as possible from the original Request deserialization
        // behavior.
        let mut request: Request = serde_json::from_str(&query_params.variables_and_extensions_as_json_string())
            .map_err(|err| serde::de::Error::custom(err.to_string()))?;
        request.operation_plan_cache_key.query = query_params.query;
        request.operation_plan_cache_key.operation_name = query_params.operation_name;
        Ok(QueryParamRequest { request })
    }
}

#[derive(serde::Deserialize)]
struct QueryParams {
    #[serde(default)]
    query: String,
    #[serde(default)]
    variables: Option<String>,
    #[serde(default)]
    operation_name: Option<String>,
    #[serde(default)]
    extensions: Option<String>,
}

impl QueryParams {
    fn variables_and_extensions_as_json_string(&self) -> String {
        let mut json = String::with_capacity(
            // {}
            2
            // at most 1 comma
            + 1
            + self.variables.as_ref().map(|v| v.len() + 12).unwrap_or_default()
            + self.extensions.as_ref().map(|e| e.len() + 13).unwrap_or_default(),
        );
        json.push('{');
        if let Some(variables) = &self.variables {
            json.push_str(&format!(r#""variables":{}"#, variables));
        }
        if let Some(extensions) = &self.extensions {
            if json.len() > 1 {
                json.push(',');
            }
            json.push_str(&format!(r#""extensions":{}"#, extensions));
        }
        json.push('}');
        json
    }
}
