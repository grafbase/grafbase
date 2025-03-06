mod binding;
mod response;

use self::{binding::*, response::*};
use grafbase_sdk::{Error, host_io::http};
use std::collections::HashMap;

impl crate::Snowflake {
    pub(crate) fn execute_statement(&self, sql: &str, bindings: &[serde_json::Value]) -> Result<Response, Error> {
        let api_url_base = self
            .config
            .snowflake_api_url_override
            .clone()
            .unwrap_or_else(|| format!("https://{}.snowflakecomputing.com", self.config.account));

        let url = format!("{api_url_base}/api/v2/statements",)
            .parse::<http::Url>()
            .map_err(|err| Error::new(err.to_string()))?;

        let mut request = http::HttpRequest::post(url);

        request.push_header("Content-Type", "application/json");
        request.push_header("Accept", "application/json");
        request.push_header("User-Agent", "grafbase-gateway");
        request.push_header("X-Snowflake-Authorization-Token-Type", "KEYPAIR_JWT");
        request.push_header("Authorization", format!("Bearer {}", self.jwt));

        let mut rendered_bindings = HashMap::with_capacity(bindings.len());

        for (index, binding) in bindings.iter().enumerate() {
            let position = index + 1;
            match binding {
                serde_json::Value::Null => {
                    return Err(Error::new(format!(
                        "Unsupported binding type: Null at position {position}",
                    )));
                }
                serde_json::Value::Bool(b) => {
                    rendered_bindings.insert(position, Binding::boolean(b.to_string()));
                }
                serde_json::Value::Number(number) => {
                    if let Some(int) = number.as_i64() {
                        rendered_bindings.insert(position, Binding::fixed(int.to_string()));
                    } else {
                        rendered_bindings.insert(position, Binding::real(number.to_string()));
                    }
                }
                serde_json::Value::String(s) => {
                    rendered_bindings.insert(position, Binding::text(s.to_string()));
                }
                serde_json::Value::Array(_) => {
                    return Err(Error::new(format!(
                        "Unsupported binding type: Array at position {position}",
                    )));
                }
                serde_json::Value::Object(_) => {
                    return Err(Error::new(format!(
                        "Unsupported binding type: Object at position {position}",
                    )));
                }
            }
        }

        let request = request.body(
            serde_json::to_vec(&Body {
                statement: sql,
                bindings: rendered_bindings,
                database: self.config.database.as_deref(),
                schema: self.config.schema.as_deref(),
                warehouse: self.config.warehouse.as_deref(),
                role: self.config.role.as_deref(),
            })
            .unwrap(),
        );

        let response = http::execute(&request).map_err(|err| Error::new(err.to_string()))?;

        // eprintln!("{}", std::str::from_utf8(response.body()).unwrap());

        let body = serde_json::from_slice(response.body()).map_err(|err| Error::new(err.to_string()))?;

        Ok(body)
    }
}

/// Reference: https://docs.snowflake.com/en/developer-guide/sql-api/reference#body-of-the-post-request-to-api-v2-statements
#[derive(serde::Serialize)]
struct Body<'a> {
    statement: &'a str,
    bindings: HashMap<usize, Binding>,

    database: Option<&'a str>,
    schema: Option<&'a str>,
    warehouse: Option<&'a str>,
    role: Option<&'a str>,
}
