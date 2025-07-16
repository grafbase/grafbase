use std::borrow::Cow;

use http::request::Parts;
use rmcp::model::CallToolResult;
use schemars::JsonSchema;
use serde::Deserialize;

use super::{SdlAndErrors, SearchTool, Tool, sdl::PartialSdl};
use crate::EngineWatcher;

pub struct IntrospectTool<R: engine::Runtime> {
    engine: EngineWatcher<R>,
}

impl<R: engine::Runtime> Tool for IntrospectTool<R> {
    type Parameters = IntrospectionParameters;

    fn name() -> &'static str {
        "introspect"
    }

    fn description(&self) -> Cow<'_, str> {
        format!("Provide the complete GraphQL SDL for the requested types. Always use `{}` first to identify relevant fields before if information on a specific type was not explicitly requested. Continue using this tool until you have the definition of all nested types you intend to query.", SearchTool::<R>::name()).into()
    }

    async fn call(&self, parts: Parts, parameters: Self::Parameters) -> anyhow::Result<CallToolResult> {
        let engine = self.engine.borrow().clone();
        let schema = engine
            .get_schema(&parts)
            .await
            .map_err(|err| anyhow::anyhow!(err.into_owned()))?;
        Ok(self.introspect(&schema, parameters.types).into())
    }

    fn annotations(&self) -> rmcp::model::ToolAnnotations {
        rmcp::model::ToolAnnotations::new().read_only(true)
    }
}

#[derive(Deserialize, JsonSchema)]
pub struct IntrospectionParameters {
    types: Vec<String>,
}

impl<R: engine::Runtime> IntrospectTool<R> {
    pub fn new(engine: &EngineWatcher<R>) -> Self {
        Self { engine: engine.clone() }
    }

    fn introspect(&self, schema: &engine::Schema, types: Vec<String>) -> SdlAndErrors {
        let mut site_ids = Vec::new();
        let mut errors = Vec::new();

        for type_name in types {
            let Some(type_definition) = schema.type_definition_by_name(&type_name) else {
                errors.push(format!("Type '{type_name}' not found"));
                continue;
            };

            site_ids.push(type_definition.id().into());
        }

        site_ids.sort_unstable();
        site_ids.dedup();

        SdlAndErrors {
            sdl: PartialSdl {
                max_depth: 2,
                search_tokens: Vec::new(),
                max_size_for_extra_content: 2048,
                site_ids_and_score: site_ids.into_iter().map(|id| (id, 1.0)).collect(),
            }
            .generate(schema),
            errors,
        }
    }
}
