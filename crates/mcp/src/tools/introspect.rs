use std::borrow::Cow;
use std::sync::Arc;

use engine_schema::Schema;
use http::request::Parts;
use rmcp::model::CallToolResult;
use schemars::JsonSchema;
use serde::Deserialize;

use super::{SdlAndErrors, Tool, sdl::PartialSdl};

pub struct IntrospectTool;

impl Tool for IntrospectTool {
    type Parameters = IntrospectionParameters;

    fn name() -> &'static str {
        "introspect"
    }

    fn description(&self) -> Cow<'_, str> {
        "Provide the complete GraphQL SDL for the requested types. Always use `search` first to identify relevant fields before if information on a specific type was not explicitly requested. Continue using this tool until you have the definition of all nested types you intend to query.".into()
    }

    async fn call(
        &self,
        _parts: Parts,
        parameters: Self::Parameters,
        schema: Arc<Schema>,
    ) -> anyhow::Result<CallToolResult> {
        Ok(Self::introspect(&schema, parameters.types).into())
    }

    fn annotations(&self) -> rmcp::model::ToolAnnotations {
        rmcp::model::ToolAnnotations::new().read_only(true)
    }
}

#[derive(Deserialize, JsonSchema)]
pub struct IntrospectionParameters {
    types: Vec<String>,
}

impl IntrospectTool {
    pub fn new() -> Self {
        Self
    }

    fn introspect(schema: &Schema, types: Vec<String>) -> SdlAndErrors {
        let mut site_ids = Vec::new();
        let mut errors = Vec::new();

        for type_name in types {
            let Some(type_definition) = schema.type_definition_by_name(&type_name) else {
                errors.push(format!("Type '{type_name}' not found").into());
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
