use std::borrow::Cow;

use rmcp::model::{CallToolResult, Content};
use serde::Serialize as _;

use super::{Request, Tool, VerifyTool};
use crate::EngineWatcher;

pub struct ExecuteTool<R: engine::Runtime> {
    engine: EngineWatcher<R>,
    #[allow(unused)]
    include_mutations: bool,
}

impl<R: engine::Runtime> Tool for ExecuteTool<R> {
    type Parameters = Request;

    fn name() -> &'static str {
        "execute"
    }

    fn description(&self) -> Cow<'_, str> {
        format!("Executes a GraphQL request and returns the response. You MUST validate a request with the `{}` tool before using this tool.", VerifyTool::<R>::name()).into()
    }

    async fn call(&self, parameters: Self::Parameters) -> anyhow::Result<CallToolResult> {
        let json = self.execute(parameters).await?;
        Ok(CallToolResult {
            content: vec![Content::text(String::from_utf8(json).unwrap())],
            is_error: None,
        })
    }
}

impl<R: engine::Runtime> ExecuteTool<R> {
    // FIXME: enforce include_mutations
    pub fn new(engine: &EngineWatcher<R>, include_mutations: bool) -> Self {
        Self {
            engine: engine.clone(),
            include_mutations,
        }
    }

    async fn execute(&self, request: Request) -> anyhow::Result<Vec<u8>> {
        let engine = self.engine.borrow().clone();
        let mut body = Vec::new();
        let mut serializer = minicbor_serde::Serializer::new(&mut body);

        // Necessary for serde_json::Value which serializes `Null` as unit rather than none...
        serializer.serialize_unit_as_null(true);
        request.serialize(&mut serializer)?;

        let http_request = http::Request::builder()
            .header("Content-Type", "application/cbor")
            .header("Accept", "application/json")
            .method(http::Method::POST)
            .body(async move { Ok(body.into()) })?;

        let response = engine.execute(http_request).await;
        Ok(response.into_body().into_bytes().unwrap().into())
    }
}
