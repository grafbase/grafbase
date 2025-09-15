mod execute;
mod introspect;
mod sdl;
mod search;

use engine_schema::Schema;
pub use execute::*;
use futures::future::BoxFuture;
use http::request::Parts;
pub use introspect::*;
pub use search::*;
use std::borrow::Cow;
use std::sync::Arc;

use rmcp::model::{CallToolResult, Content, ErrorCode, ErrorData, JsonObject, ToolAnnotations};

pub(crate) trait Tool: Send + Sync + 'static {
    type Parameters: serde::de::DeserializeOwned + schemars::JsonSchema;
    fn name() -> &'static str;
    fn description(&self) -> Cow<'_, str>;
    fn call(
        &self,
        parts: Parts,
        parameters: Self::Parameters,
        schema: Arc<Schema>,
    ) -> impl Future<Output = anyhow::Result<CallToolResult>> + Send;
    fn annotations(&self) -> ToolAnnotations;
}

pub(crate) trait RmcpTool: Send + Sync + 'static {
    fn name(&self) -> &str;
    fn to_tool(&self) -> rmcp::model::Tool;
    fn call(
        &self,
        parts: Parts,
        parameters: Option<JsonObject>,
        schema: Arc<Schema>,
    ) -> BoxFuture<'_, Result<CallToolResult, ErrorData>>;
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
            .annotate(self.annotations())
    }

    fn call(
        &self,
        parts: Parts,
        parameters: Option<JsonObject>,
        schema: Arc<Schema>,
    ) -> BoxFuture<'_, Result<CallToolResult, ErrorData>> {
        Box::pin(async move {
            let parameters: T::Parameters =
                serde_json::from_value(serde_json::Value::Object(parameters.unwrap_or_default()))
                    .map_err(|err| ErrorData::new(ErrorCode::INVALID_PARAMS, err.to_string(), None))?;
            match Tool::call(self, parts, parameters, schema).await {
                Ok(data) => Ok(data),
                Err(err) => Err(ErrorData::new(ErrorCode::INTERNAL_ERROR, err.to_string(), None)),
            }
        })
    }
}

struct SdlAndErrors {
    sdl: String,
    errors: Vec<Cow<'static, str>>,
}

impl From<SdlAndErrors> for CallToolResult {
    fn from(SdlAndErrors { sdl, errors }: SdlAndErrors) -> Self {
        let out = if !errors.is_empty() {
            const ERROR_TITLE: &str = "Errors:\n";
            const SDL_TITLE: &str = "\n== GraphQL SDL ==\n";
            let mut out = String::with_capacity(
                SDL_TITLE.len() + ERROR_TITLE.len() + sdl.len() + errors.iter().map(|err| err.len() + 1).sum::<usize>(),
            );
            out.push_str(ERROR_TITLE);
            for err in &errors {
                out.push_str(err);
                out.push('\n');
            }
            if !sdl.is_empty() {
                out.push_str(SDL_TITLE);
                out.push_str(&sdl);
            }
            out
        } else {
            sdl
        };
        CallToolResult {
            content: Some(vec![Content::text(out)]),
            structured_content: None,
            is_error: None,
        }
    }
}
