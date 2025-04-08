use std::{borrow::Cow, sync::Arc};

use engine::{
    Schema,
    mcp::{McpRequestContext, McpResponseExtension},
};
use engine_operation::RawVariables;
use rmcp::model::{CallToolResult, Content};
use serde::Serialize as _;

use super::{Tool, sdl::PartialSdl};
use crate::EngineWatcher;

pub struct ExecuteTool<R: engine::Runtime> {
    engine: EngineWatcher<R>,
    #[allow(unused)]
    include_mutations: bool,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Request {
    pub query: String,
    pub variables: RawVariables,
}

impl schemars::JsonSchema for Request {
    fn schema_name() -> std::string::String {
        "Request".to_owned()
    }
    fn schema_id() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed(std::concat!(std::module_path!(), "::", "Request"))
    }
    fn json_schema(generator: &mut schemars::r#gen::SchemaGenerator) -> schemars::schema::Schema {
        {
            let mut schema_object = schemars::schema::SchemaObject {
                instance_type: Some(schemars::schema::InstanceType::Object.into()),
                ..Default::default()
            };
            let object_validation = schema_object.object();
            {
                schemars::_private::insert_object_property::<String>(
                    object_validation,
                    "query",
                    false,
                    false,
                    generator.subschema_for::<String>(),
                );
            }
            {
                schemars::_private::insert_object_property::<serde_json::Map<String, serde_json::Value>>(
                    object_validation,
                    "variables",
                    false,
                    false,
                    generator.subschema_for::<serde_json::Map<String, serde_json::Value>>(),
                );
            }
            schemars::schema::Schema::Object(schema_object)
        }
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

    async fn call(&self, parameters: Self::Parameters) -> anyhow::Result<CallToolResult> {
        let EngineResponse { schema, json, mcp } = self.execute(parameters).await?;
        let mut content = vec![Content::text(String::from_utf8(json).unwrap())];
        if let Some(McpResponseExtension { mut site_ids }) = mcp {
            if !site_ids.is_empty() {
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
        }
        Ok(CallToolResult {
            content,
            is_error: None,
        })
    }
}

struct EngineResponse {
    schema: Arc<Schema>,
    json: Vec<u8>,
    mcp: Option<McpResponseExtension>,
}

impl<R: engine::Runtime> ExecuteTool<R> {
    // FIXME: enforce include_mutations
    pub fn new(engine: &EngineWatcher<R>, include_mutations: bool) -> Self {
        Self {
            engine: engine.clone(),
            include_mutations,
        }
    }

    async fn execute(&self, request: Request) -> anyhow::Result<EngineResponse> {
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
            .extension(McpRequestContext {
                include_mutations: self.include_mutations,
            })
            .body(async move { Ok(body.into()) })?;

        let mut response = engine.execute(http_request).await;
        let mcp = response.extensions_mut().remove();
        Ok(EngineResponse {
            schema: engine.schema.clone(),
            json: response.into_body().into_bytes().unwrap().into(),
            mcp,
        })
    }
}
