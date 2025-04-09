use std::borrow::Cow;

use engine_operation::{Operation, RawVariables, Variables};
use rmcp::model::CallToolResult;
use serde::{Deserialize, Serialize};

use super::{ExecuteTool, SdlAndErrors, Tool, sdl::PartialSdl};
use crate::EngineWatcher;

pub struct VerifyTool<R: engine::Runtime> {
    engine: EngineWatcher<R>,
    include_mutations: bool,
}

impl<R: engine::Runtime> Tool for VerifyTool<R> {
    type Parameters = Request;

    fn name() -> &'static str {
        "verify"
    }

    fn description(&self) -> Cow<'_, str> {
        format!(
            "Validates a GraphQL request. You MUST call this tool before `{}`",
            ExecuteTool::<R>::name()
        )
        .into()
    }

    async fn call(&self, parameters: Self::Parameters) -> anyhow::Result<CallToolResult> {
        Ok(self.validate_request(parameters).into())
    }
}

#[derive(Deserialize, Serialize)]
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

impl<R: engine::Runtime> VerifyTool<R> {
    pub fn new(engine: &EngineWatcher<R>, include_mutations: bool) -> Self {
        Self {
            engine: engine.clone(),
            include_mutations,
        }
    }

    fn validate_request(&self, request: Request) -> SdlAndErrors {
        let schema = self.engine.borrow().schema.clone();

        let operation = match Operation::parse(&schema, None, &request.query) {
            Ok(op) => op,
            Err(engine_operation::Errors { items, .. }) => {
                let mut errors = Vec::new();
                let mut site_ids = Vec::new();
                for err in items {
                    errors.push(err.message.into_owned());
                    if let Some(site_id) = err.site_id {
                        site_ids.push(site_id);
                    }
                }
                site_ids.sort_unstable();
                site_ids.dedup();

                let sdl = PartialSdl {
                    max_depth: 2,
                    search_tokens: Vec::new(),
                    max_size_for_extra_content: 1024,
                    site_ids_and_score: site_ids.into_iter().map(|id| (id, 1.0)).collect(),
                }
                .generate(&schema);

                return SdlAndErrors { errors, sdl };
            }
        };

        if operation.attributes.ty.is_mutation() && !self.include_mutations {
            return SdlAndErrors {
                errors: vec!["Mutaions are not allowed".to_string()],
                sdl: String::new(),
            };
        }

        let mut errors = Vec::new();
        match Variables::bind(&schema, &operation, request.variables) {
            Ok(variables) => {
                if let Err(complexity_err) = operation.compute_and_validate_complexity(&schema, &variables) {
                    errors.push(complexity_err.to_string());
                }
            }
            Err(var_errors) => {
                errors.extend(var_errors.into_iter().map(|e| e.to_string()));
            }
        }

        SdlAndErrors {
            errors,
            sdl: String::new(),
        }
    }
}
