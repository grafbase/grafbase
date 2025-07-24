use crate::InstanceState;

pub use super::grafbase::sdk::http_types::*;

impl Host for InstanceState {}

impl TryFrom<&http::Method> for HttpMethod {
    type Error = wasmtime::Error;

    fn try_from(method: &http::Method) -> wasmtime::Result<Self> {
        Ok(match method.as_str() {
            "GET" => HttpMethod::Get,
            "POST" => HttpMethod::Post,
            "PUT" => HttpMethod::Put,
            "DELETE" => HttpMethod::Delete,
            "PATCH" => HttpMethod::Patch,
            "HEAD" => HttpMethod::Head,
            "OPTIONS" => HttpMethod::Options,
            _ => {
                return Err(wasmtime::Error::msg(format!("Invalid HTTP method: {method}")));
            }
        })
    }
}
