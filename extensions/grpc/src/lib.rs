use grafbase_sdk::{
    dynamic::Request,
    http::{HeaderMap, HeaderName, HeaderValue, Method},
    resolver, Response,
};
use serde_json::{json, Value};

#[resolver("grpc")]
async fn resolve(ctx: Context<Value>) -> Result<Value, Error> {
    let endpoint = ctx.directive.argument("endpoint")?;
    let service = ctx.directive.argument("service")?;
    let method = ctx.directive.argument("method")?;
    let request_template: Option<Value> = ctx.directive.argument("request").ok();
    let response_path: Option<String> = ctx.directive.argument("responsePath").ok();
    let timeout: Option<i64> = ctx.directive.argument("timeout").ok();
    let headers: Option<Value> = ctx.directive.argument("headers").ok();

    // Prepare request payload
    let request_payload = if let Some(template) = request_template {
        interpolate_variables(template, &ctx.arguments)
    } else {
        json!({})
    };

    // Prepare headers
    let mut request_headers = HeaderMap::new();
    request_headers.insert(
        HeaderName::from_static("content-type"),
        HeaderValue::from_static("application/grpc+json"),
    );

    if let Some(Value::Object(headers)) = headers {
        for (key, value) in headers {
            if let Value::String(value) = value {
                if let Ok(header_name) = HeaderName::try_from(key.as_str()) {
                    if let Ok(header_value) = HeaderValue::try_from(value.as_str()) {
                        request_headers.insert(header_name, header_value);
                    }
                }
            }
        }
    }

    // Build the URL
    let url = format!(
        "{}/{}{}",
        endpoint.trim_end_matches('/'),
        service,
        if method.starts_with('/') {
            method
        } else {
            format!("/{}", method)
        }
    );

    // Create and send request
    let mut request = Request::new(Method::POST, &url);
    request.headers = request_headers;
    request.body = Some(serde_json::to_string(&request_payload)?);

    if let Some(timeout_ms) = timeout {
        request.timeout = std::time::Duration::from_millis(timeout_ms as u64);
    }

    let response = request.send().await?;
    
    if !response.status.is_success() {
        return Err(format!(
            "gRPC request failed with status: {}",
            response.status
        ).into());
    }

    let response_body: Value = serde_json::from_slice(&response.body)?;

    // Extract response data using path if specified
    if let Some(path) = response_path {
        extract_path_value(&response_body, &path)
            .ok_or_else(|| Error::new("Failed to extract response path"))
    } else {
        Ok(response_body)
    }
}

fn interpolate_variables(template: Value, arguments: &Value) -> Value {
    match template {
        Value::Object(map) => {
            let mut result = serde_json::Map::new();
            for (key, value) in map {
                result.insert(key, interpolate_variables(value, arguments));
            }
            Value::Object(result)
        }
        Value::Array(arr) => {
            Value::Array(arr.into_iter().map(|v| interpolate_variables(v, arguments)).collect())
        }
        Value::String(s) => {
            if s.starts_with("{{") && s.ends_with("}}") {
                let var_name = s[2..s.len()-2].trim();
                if let Some(value) = arguments.get(var_name) {
                    value.clone()
                } else {
                    Value::String(s)
                }
            } else {
                Value::String(s)
            }
        }
        _ => template,
    }
}

fn extract_path_value(value: &Value, path: &str) -> Option<Value> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = value;
    
    for part in parts {
        current = current.get(part)?;
    }
    
    Some(current.clone())
} 