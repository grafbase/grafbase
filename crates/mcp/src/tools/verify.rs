use std::borrow::Cow;

use engine_operation::{Operation, RawVariables, Variables};
use rmcp::model::CallToolResult;
use serde::{Deserialize, Serialize};

use super::Tool;
use crate::EngineWatcher;

pub struct VerifyTool<R: engine::Runtime> {
    engine: EngineWatcher<R>,
    enable_mutations: bool,
}

impl<R: engine::Runtime> Tool for VerifyTool<R> {
    type Parameters = Request;

    fn name() -> &'static str {
        "verify"
    }

    fn description(&self) -> Cow<'_, str> {
        "Validates a GraphQL request. A list of errors is returned if there are any.".into()
    }

    async fn call(&self, parameters: Self::Parameters) -> anyhow::Result<CallToolResult> {
        let errors = self.validate_request(parameters);
        Ok(CallToolResult {
            content: vec![rmcp::model::Content::json(&errors)?],
            is_error: Some(!errors.is_empty()),
        })
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

#[derive(Serialize)]
pub struct VerifyResponse {
    errors: Vec<String>,
}

impl<R: engine::Runtime> VerifyTool<R> {
    pub fn new(engine: &EngineWatcher<R>, enable_mutations: bool) -> Self {
        Self {
            engine: engine.clone(),
            enable_mutations,
        }
    }

    fn validate_request(&self, request: Request) -> Vec<String> {
        let mut errors = Vec::new();
        let schema = self.engine.borrow().schema.clone();

        let operation = match Operation::parse(&schema, None, &request.query) {
            Ok(op) => op,
            Err(err) => {
                return vec![err.to_string()];
            }
        };

        if operation.attributes.ty.is_mutation() && !self.enable_mutations {
            return vec!["Mutaions are not allowed".to_string()];
        }

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

        errors
    }
}
