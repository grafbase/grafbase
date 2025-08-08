use std::borrow::Cow;

use engine::mcp::{McpRequestContext, McpResponseExtension};
use engine_operation::RawVariables;
use http::request::Parts;
use rmcp::model::{CallToolResult, Content};
use serde::Serialize as _;

use super::{Tool, sdl::PartialSdl};
use crate::EngineWatcher;

pub struct ExecuteTool<R: engine::Runtime> {
    engine: EngineWatcher<R>,
    execute_mutations: bool,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Request {
    pub query: String,
    // Note: accept empty variables, in case the LLM fails to send variables, although they are required.
    #[serde(default)]
    pub variables: RawVariables,
}

impl schemars::JsonSchema for Request {
    fn schema_name() -> schemars::_private::alloc::borrow::Cow<'static, str> {
        schemars::_private::alloc::borrow::Cow::Borrowed("Request")
    }
    fn schema_id() -> schemars::_private::alloc::borrow::Cow<'static, str> {
        schemars::_private::alloc::borrow::Cow::Borrowed(::core::concat!(::core::module_path!(), "::", "Request"))
    }
    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        {
            let mut schema = schemars::json_schema!({
                "type":"object",
            });
            {
                schemars::_private::insert_object_property(&mut schema, "query", false, {
                    generator.subschema_for::<String>()
                });
            }
            {
                schemars::_private::insert_object_property(&mut schema, "variables", true, {
                    let mut schema = generator.subschema_for::<serde_json::Map<String, serde_json::Value>>();
                    schema.insert("default".into(), serde_json::json!({}));
                    schema
                });
            }
            schema
        }
    }
    fn inline_schema() -> bool {
        false
    }
}

impl<R: engine::Runtime> Tool for ExecuteTool<R> {
    type Parameters = Request;

    fn name() -> &'static str {
        "execute"
    }

    fn description(&self) -> Cow<'_, str> {
        "Executes a GraphQL request. Additional GraphQL SDL may be provided upon errors.".into()
    }

    async fn call(&self, parts: Parts, parameters: Self::Parameters) -> anyhow::Result<CallToolResult> {
        let EngineResponse { json, mcp } = self.execute(parts, parameters).await?;
        let mut content = vec![Content::text(String::from_utf8(json).unwrap())];
        if let Some(McpResponseExtension { schema, mut site_ids }) = mcp
            && !site_ids.is_empty()
        {
            site_ids.sort_unstable();
            site_ids.dedup();

            let sdl = PartialSdl {
                max_depth: 2,
                search_tokens: Vec::new(),
                max_size_for_extra_content: 1024,
                site_ids_and_score: site_ids.into_iter().map(|id| (id, 1.0)).collect(),
            }
            .generate(&schema);
            content.push(Content::text(sdl));
        }
        Ok(CallToolResult {
            content,
            is_error: None,
        })
    }

    fn annotations(&self) -> rmcp::model::ToolAnnotations {
        rmcp::model::ToolAnnotations::new().destructive(true).open_world(true)
    }
}

struct EngineResponse {
    json: Vec<u8>,
    mcp: Option<McpResponseExtension>,
}

impl<R: engine::Runtime> ExecuteTool<R> {
    pub fn new(engine: &EngineWatcher<R>, execute_mutations: bool) -> Self {
        Self {
            engine: engine.clone(),
            execute_mutations,
        }
    }

    async fn execute(&self, mut parts: Parts, request: Request) -> anyhow::Result<EngineResponse> {
        let engine = self.engine.borrow().clone();
        let mut body = Vec::new();
        let mut serializer = minicbor_serde::Serializer::new(&mut body);

        // Necessary for serde_json::Value which serializes `Null` as unit rather than none...
        serializer.serialize_unit_as_null(true);
        request.serialize(&mut serializer)?;
        let body = async move { Ok(body.into()) };

        parts.method = http::Method::POST;
        parts
            .headers
            .insert("Content-Type", http::HeaderValue::from_static("application/cbor"));
        parts
            .headers
            .insert("Accept", http::HeaderValue::from_static("application/json"));
        parts.extensions.insert(McpRequestContext {
            execute_mutations: self.execute_mutations,
        });

        let http_request = http::Request::from_parts(parts, body);
        let mut response = engine.execute(http_request).await;
        let mcp = response.extensions_mut().remove();
        Ok(EngineResponse {
            json: response.into_body().into_bytes().unwrap().into(),
            mcp,
        })
    }
}
