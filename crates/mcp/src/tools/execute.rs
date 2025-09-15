use std::borrow::Cow;
use std::sync::Arc;

use axum::body::Bytes;
use engine_operation::{self as operation, Operation, RawVariables, Variables};
use engine_schema::Schema;
use http::request::Parts;
use rmcp::model::{CallToolResult, Content};

use crate::GraphQLServer;

use super::{SdlAndErrors, Tool, sdl::PartialSdl};

pub struct ExecuteTool<G: GraphQLServer> {
    gql: G,
    can_mutate: bool,
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

impl<R: GraphQLServer> Tool for ExecuteTool<R> {
    type Parameters = Request;

    fn name() -> &'static str {
        "execute"
    }

    fn description(&self) -> Cow<'_, str> {
        "Executes a GraphQL request. Additional GraphQL SDL may be provided upon errors.".into()
    }

    async fn call(
        &self,
        parts: Parts,
        request: Self::Parameters,
        schema: Arc<Schema>,
    ) -> anyhow::Result<CallToolResult> {
        // Parse the operation first to check for errors
        let operation = match Operation::parse(&schema, None, &request.query) {
            Ok(operation) => operation,
            Err(operation::Errors { items, .. }) => return Ok(process_operation_errors(&schema, items).into()),
        };

        // Check if it's a subscription - never supported
        if operation.attributes.ty.is_subscription() {
            return Ok(SdlAndErrors {
                sdl: String::new(),
                errors: vec!["Subscriptions are not supported through MCP.".into()],
            }
            .into());
        }

        // Check if it's a mutation and whether mutations are allowed
        if operation.attributes.ty.is_mutation() && !self.can_mutate {
            return Ok(SdlAndErrors {
                sdl: String::new(),
                errors: vec!["Mutations are not allowed.".into()],
            }
            .into());
        }

        // Bind and validate variables
        if let Err(errors) = Variables::bind(&schema, &operation, request.variables.clone()) {
            return Ok(process_operation_errors(&schema, errors).into());
        }

        // If we get here, the operation is valid - forward it to the runtime
        let response = self.execute(parts, request).await?;

        Ok(CallToolResult {
            content: Some(vec![Content::text(
                String::from_utf8(response.to_vec()).unwrap_or_else(|_| String::from("Not an UTF-8 response.")),
            )]),
            structured_content: None,
            is_error: None,
        })
    }

    fn annotations(&self) -> rmcp::model::ToolAnnotations {
        rmcp::model::ToolAnnotations::new().destructive(true).open_world(true)
    }
}

impl<G: GraphQLServer> ExecuteTool<G> {
    pub fn new(gql: G, can_mutate: bool) -> Self {
        Self { gql, can_mutate }
    }

    async fn execute(&self, mut parts: Parts, request: Request) -> anyhow::Result<Bytes> {
        // Serialize request to JSON
        let json_body = serde_json::to_vec(&request)?;
        let content_length = json_body.len();

        // Create a new Parts with proper headers for JSON request
        parts.headers.insert(
            http::header::CONTENT_TYPE,
            http::HeaderValue::from_static("application/json"),
        );
        parts.headers.insert(
            http::header::CONTENT_LENGTH,
            http::HeaderValue::from_str(&content_length.to_string())?,
        );
        parts
            .headers
            .insert(http::header::ACCEPT, http::HeaderValue::from_static("application/json"));

        // Forward request to runtime
        self.gql.execute(parts, axum::body::Bytes::from(json_body)).await
    }
}

fn process_operation_errors(schema: &Schema, errors: Vec<operation::Error>) -> SdlAndErrors {
    // Collect error site_ids for SDL generation
    let mut site_ids: Vec<_> = errors.iter().filter_map(|error| error.site_id).collect();
    site_ids.sort_unstable();
    site_ids.dedup();

    // Generate error messages
    let errors = errors.into_iter().map(error_to_string).collect();

    // Generate SDL for error locations
    let sdl = if !site_ids.is_empty() {
        PartialSdl {
            max_depth: 2,
            search_tokens: Vec::new(),
            max_size_for_extra_content: 2048,
            site_ids_and_score: site_ids.into_iter().map(|id| (id, 1.0)).collect(),
        }
        .generate(schema)
    } else {
        String::new()
    };

    SdlAndErrors { sdl, errors }
}

fn error_to_string(error: operation::Error) -> Cow<'static, str> {
    use std::fmt::Write;

    let n = error.locations.len();
    let mut locations = error.locations.into_iter();
    if let Some(location) = locations.next() {
        let mut out = String::with_capacity(error.message.len() + 3 + n * 6);
        out.push_str("At ");
        write!(&mut out, "{location}").unwrap();
        for location in locations {
            write!(&mut out, ", {location}").unwrap();
        }
        out.push(' ');
        out.push_str(&error.message);
        out.into()
    } else {
        error.message
    }
}
