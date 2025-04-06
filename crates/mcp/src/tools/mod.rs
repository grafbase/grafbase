#![allow(refining_impl_trait)]
mod execute;
mod introspect;
mod sdl;
mod search;
mod verify;

pub use execute::*;
use futures::future::BoxFuture;
pub use introspect::*;
pub use search::*;
use std::borrow::Cow;
pub use verify::*;

use rmcp::model::{CallToolResult, Content, ErrorCode, ErrorData, JsonObject};

pub trait Tool: Send + Sync + 'static {
    type Parameters: serde::de::DeserializeOwned + schemars::JsonSchema;
    fn name() -> &'static str;
    fn description(&self) -> Cow<'_, str>;
    fn call(&self, parameters: Self::Parameters) -> impl Future<Output = anyhow::Result<CallToolResult>> + Send;
}

pub trait RmcpTool: Send + Sync + 'static {
    fn name(&self) -> &str;
    fn to_tool(&self) -> rmcp::model::Tool;
    fn call(&self, parameters: Option<JsonObject>) -> BoxFuture<'_, Result<CallToolResult, ErrorData>>;
}

impl<T: Tool> RmcpTool for T {
    fn name(&self) -> &str {
        T::name()
    }

    fn to_tool(&self) -> rmcp::model::Tool {
        let serde_json::Value::Object(schema) =
            serde_json::to_value(schemars::schema_for!(<T as Tool>::Parameters)).unwrap()
        else {
            unreachable!()
        };
        rmcp::model::Tool::new(self.name().to_string(), self.description().into_owned(), schema)
    }

    fn call(&self, parameters: Option<JsonObject>) -> BoxFuture<'_, Result<CallToolResult, ErrorData>> {
        Box::pin(async move {
            let parameters: T::Parameters =
                serde_json::from_value(serde_json::Value::Object(parameters.unwrap_or_default()))
                    .map_err(|err| ErrorData::new(ErrorCode::INVALID_PARAMS, err.to_string(), None))?;
            match Tool::call(self, parameters).await {
                Ok(data) => Ok(data),
                Err(err) => Err(ErrorData::new(ErrorCode::INTERNAL_ERROR, err.to_string(), None)),
            }
        })
    }
}

struct SdlAndErrors {
    sdl: String,
    errors: Vec<String>,
}

impl From<SdlAndErrors> for CallToolResult {
    fn from(SdlAndErrors { sdl, errors }: SdlAndErrors) -> Self {
        let mut content = Vec::new();
        if !sdl.is_empty() {
            content.push(Content::text(sdl));
        }
        if !errors.is_empty() {
            content.push(Content::json(ErrorList { errors }).unwrap());
        }
        CallToolResult {
            content,
            is_error: None,
        }
    }
}

#[derive(serde::Serialize)]
struct ErrorList<T> {
    errors: Vec<T>,
}
