mod search;

use futures::future::BoxFuture;
pub use search::SearchTool;
use std::borrow::Cow;

use rmcp::model::{CallToolResult, Content, ErrorCode, ErrorData, JsonObject};

pub trait Tool: Send + Sync + 'static {
    type Parameters: serde::de::DeserializeOwned + schemars::JsonSchema;
    type Response: serde::Serialize;
    type Error: serde::Serialize;
    fn name(&self) -> &str;
    fn description(&self) -> Cow<'_, str>;
    fn call(
        &self,
        parameters: Self::Parameters,
    ) -> impl Future<Output = anyhow::Result<Result<Self::Response, Self::Error>>> + Send;
}

pub trait RmcpTool: Send + Sync + 'static {
    fn name(&self) -> &str;
    fn to_tool(&self) -> rmcp::model::Tool;
    fn call(&self, parameters: Option<JsonObject>) -> BoxFuture<'_, Result<CallToolResult, ErrorData>>;
}

impl<T: Tool> RmcpTool for T {
    fn name(&self) -> &str {
        Tool::name(self)
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
            Tool::call(self, parameters)
                .await
                .map(|result| match result {
                    Ok(data) => CallToolResult {
                        content: vec![Content::json(data).unwrap()],
                        is_error: Some(false),
                    },
                    Err(data) => CallToolResult {
                        content: vec![Content::json(data).unwrap()],
                        is_error: Some(true),
                    },
                })
                .map_err(|err| ErrorData::new(ErrorCode::INTERNAL_ERROR, err.to_string(), None))
        })
    }
}
