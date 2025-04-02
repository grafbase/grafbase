use std::{borrow::Cow, fmt};

use axum::body::Body;
use engine::Runtime;
use engine_schema::{Definition, InputObjectDefinition, ScalarType};
use http::{
    Method, Request,
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
};
use indoc::{formatdoc, indoc};
use rmcp::{
    RoleServer, ServerHandler,
    model::{
        CallToolRequestParam, CallToolResult, Content, ErrorData, Implementation, ListToolsResult,
        PaginatedRequestParam, ProtocolVersion, ServerCapabilities, ServerInfo, Tool,
    },
    service::RequestContext,
};
use serde_json::{Map, Value, json};

use crate::server::gateway::EngineWatcher;

pub struct McpServer<R: Runtime> {
    info: ServerInfo,
    engine: EngineWatcher<R>,
    auth: Option<String>,
    mutations_enabled: bool,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IntrospectType {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    fields: Option<Vec<IntrospectField>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    input_fields: Option<Vec<IntrospectArgument>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    interfaces: Option<Vec<IntrospectType>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    possible_types: Option<Vec<IntrospectType>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    of_type: Option<Box<IntrospectType>>,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IntrospectField {
    name: String,
    r#type: Box<IntrospectType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    is_deprecated: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    deprecation_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    args: Option<Vec<IntrospectArgument>>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntrospectArgument {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    r#type: Box<IntrospectType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    default_value: Option<String>,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct IntrospectFieldWrapper {
    #[serde(rename = "__type")]
    r#type: Option<IntrospectType>,
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryResponse<T> {
    data: Option<T>,
    errors: Option<Vec<QueryError>>,
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryError {
    message: String,
}

impl<R: Runtime> McpServer<R> {
    pub fn new(
        engine: EngineWatcher<R>,
        instructions: Option<String>,
        auth: Option<String>,
        enable_mutations: bool,
    ) -> Self {
        let guide = indoc! {r#"
            This is a GraphQL server that provides tools to access certain selected queries. The queries are
            prefixed with query- and followed by the name of the query. The query requires certain arguments,
            and always a selection. You can construct the correct selection by first looking into the description
            of the query tool, finding the return type, and then calling the introspect-type tool with the name of the type.

            This tool will provide you all the information to construct a correct selection for the query. You always have to
            call the introspect-type tool first, and only after that you can call the correct query tool.
        "#};

        let instructions = match instructions {
            Some(instructions) => format!("{instructions}\n\n{guide}"),
            None => guide.to_string(),
        };

        Self {
            info: ServerInfo {
                protocol_version: ProtocolVersion::V_2024_11_05,
                capabilities: ServerCapabilities::builder().enable_tools().build(),
                server_info: Implementation::from_build_env(),
                instructions: Some(instructions),
            },
            engine,
            auth,
            mutations_enabled: enable_mutations,
        }
    }

    async fn introspect_type(&self, type_name: &str) -> Result<IntrospectType, ErrorData> {
        let query = indoc! {r#"
            query McpIntrospectType($name: String!) {
              __type(name: $name) {
                name
                kind
                description
                inputFields {
                  name
                  description
                  defaultValue
                  type {
                    ...TypeRef
                  }
                }
                fields {
                  name
                  description
                  isDeprecated
                  deprecationReason
                  args {
                    name
                    description
                    defaultValue
                    type {
                      ...TypeRef
                    }
                  }
                  type {
                    ...TypeRef
                  }
                }
                interfaces {
                  ...TypeRef
                }
                possibleTypes {
                  ...TypeRef
                }
                ofType {
                  ...TypeRef
                }
              }
            }

            fragment TypeRef on __Type {
              name
              kind
              description
              fields {
                name
                description
                isDeprecated
                deprecationReason
                args {
                  name
                  description
                  defaultValue
                  type {
                    name
                    kind
                    description
                    ofType {
                      name
                      kind
                      description
                    }
                  }
                }
                type {
                  name
                  kind
                  description
                  ofType {
                    name
                    kind
                    description
                  }
                }
              }
              interfaces {
                name
                kind
                description
              }
              possibleTypes {
                name
                kind
                description
              }
              ofType {
                name
                kind
                description
              }
            }
        "#};

        let body = json!({
            "query": query,
            "variables": {
                "name": type_name,
            },
        });

        let mut builder = Request::builder()
            .method(Method::POST)
            .header(CONTENT_TYPE, "application/json")
            .header(ACCEPT, "application/json");

        if let Some(token) = self.auth.as_ref() {
            builder = builder.header(AUTHORIZATION, format!("Bearer {token}"));
        }

        let request = builder
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .map_err(|err| ErrorData::internal_error(err.to_string(), None))?;

        let engine = self.engine.borrow().clone();
        let response = engine_axum::execute(engine, request, usize::MAX).await;

        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .map_err(|err| ErrorData::internal_error(err.to_string(), None))?;

        let response: QueryResponse<IntrospectFieldWrapper> =
            serde_json::from_slice(bytes.as_ref()).map_err(|err| ErrorData::internal_error(err.to_string(), None))?;

        match (response.data.and_then(|d| d.r#type), response.errors) {
            (_, Some(errors)) if !errors.is_empty() => {
                let message = errors
                    .into_iter()
                    .map(|error| error.message)
                    .collect::<Vec<_>>()
                    .join(", ");

                Err(ErrorData::invalid_request(message, None))
            }
            (Some(data), _) => Ok(data),
            (None, _) => Err(ErrorData::invalid_params("Type not found", None)),
        }
    }

    async fn execute_query(
        &self,
        query_name: &str,
        selection_set: &str,
        variables: Map<String, Value>,
    ) -> Result<Value, ErrorData> {
        let arguments = if !variables.is_empty() {
            Cow::Owned(format!("({})", variables_to_string(variables)))
        } else {
            Cow::Borrowed("")
        };

        let query = format!(r#"query {{ {query_name}{arguments} {selection_set} }}"#);

        let body = json!({
            "query": query,
            "variables": {},
        });

        let mut builder = Request::builder()
            .method(Method::POST)
            .header(CONTENT_TYPE, "application/json")
            .header(ACCEPT, "application/json");

        if let Some(token) = self.auth.as_ref() {
            builder = builder.header(AUTHORIZATION, format!("Bearer {}", token));
        }

        let request = builder
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .map_err(|err| ErrorData::internal_error(err.to_string(), None))?;

        let engine = self.engine.borrow().clone();
        let response = engine_axum::execute(engine, request, usize::MAX).await;

        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .map_err(|err| ErrorData::internal_error(err.to_string(), None))?;

        let response: QueryResponse<serde_json::Map<String, serde_json::Value>> =
            serde_json::from_slice(bytes.as_ref()).map_err(|err| ErrorData::internal_error(err.to_string(), None))?;

        match (response.data.and_then(|mut d| d.remove(query_name)), response.errors) {
            (_, Some(errors)) if !errors.is_empty() => {
                let message = errors
                    .into_iter()
                    .map(|error| error.message)
                    .collect::<Vec<_>>()
                    .join(", ");

                Err(ErrorData::invalid_request(message, None))
            }
            (Some(data), _) => Ok(data),
            (None, _) => Err(ErrorData::internal_error("Invalid query response", None)),
        }
    }
}

fn variables_to_string(variables: Map<String, Value>) -> String {
    variables
        .into_iter()
        .map(|(name, value)| match value {
            Value::Null => format!("{name}: null"),
            Value::Bool(val) => format!("{name}: {val}"),
            Value::Number(number) => format!("{name}: {number}"),
            Value::String(s) => format!("{name}: \"{s}\""),
            v @ Value::Array(_) => format!("{name}: {v}"),
            Value::Object(map) => format!("{name}: {{ {} }}", variables_to_string(map)),
        })
        .collect::<Vec<_>>()
        .join(", ")
}

impl<R: Runtime> ServerHandler for McpServer<R> {
    fn get_info(&self) -> ServerInfo {
        self.info.clone()
    }

    async fn list_tools(
        &self,
        _: PaginatedRequestParam,
        _: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        let mut tools = Vec::new();
        let engine = self.engine.borrow().clone();

        let description = indoc! {r#"
           Use this tool before executing any query tools. This tool provides information how to construct
           a selection for a specific query. You first select a query you want to execute, see its return
           type from the description, use this tool to get information about the type and _only then_ you
           call the query tool with the correct selection set and arguments.

           Remember, THIS IS IMPORTANT: you can ONLY select the fields that are returned by this query. There
           are no other fields that can be selected.

           You don't need to use this API for scalar types, input types or enum values, but only when you need
           to build a selection set for a query or mutation. Use the returned value to build a selection set.
           If a field of a type is either object, interface, or union, you can call this tool repeatedly with
           the name of the type to introspect its fields.

           If the type is an object, it will have fields defined that you can use as the selection.
           The fields might have arguments, and if they are required, you need to provide them in the
           selection set.

           If the type is an interface or a union, it will have only the fields that are defined in the
           interface. You can check the possibleTypes of the interface to see what fields you can use for
           each possible type. Remember to use fragment syntax to select fields from the possible types.
        "#};

        tools.push(Tool::new(
            "introspect-type",
            description,
            json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "The name of the type, interface, or union to introspect."
                    }
                },
                "required": ["name"]
            })
            .as_object()
            .unwrap()
            .clone(),
        ));

        for field in engine.schema().query().fields() {
            if field.name() == "__type" || field.name() == "__schema" {
                continue;
            }

            add_field_to_tools(ToolType::Query, &mut tools, field);
        }

        match engine.schema().mutation() {
            Some(mutation) if self.mutations_enabled => {
                for field in mutation.fields() {
                    add_field_to_tools(ToolType::Mutation, &mut tools, field);
                }
            }
            _ => {}
        }

        Ok(ListToolsResult {
            next_cursor: None,
            tools,
        })
    }

    async fn call_tool(
        &self,
        CallToolRequestParam { name, arguments }: CallToolRequestParam,
        _: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let content = match &*name {
            "introspect-type" => {
                let Some(mut arguments) = arguments else {
                    return Err(ErrorData::invalid_params("Missing arguments", None));
                };

                let name_value = arguments
                    .remove("name")
                    .ok_or_else(|| ErrorData::invalid_params("Missing 'name' argument", None))?;

                let name = name_value
                    .as_str()
                    .ok_or_else(|| ErrorData::invalid_params("'name' argument must be a string", None))?
                    .to_string();

                let result = self.introspect_type(&name).await?;

                Content::json(result)?
            }
            command if command.starts_with("query-") => {
                let Some(mut arguments) = arguments else {
                    return Err(ErrorData::invalid_params("Missing arguments", None));
                };

                let Some(selection_set) = arguments
                    .remove("__selection")
                    .and_then(|s| s.as_str().map(ToString::to_string))
                else {
                    return Err(ErrorData::invalid_params("Missing '__selection' argument", None));
                };

                let query_name = command
                    .strip_prefix("query-")
                    .ok_or_else(|| ErrorData::invalid_params("Query name must be a string", None))?;

                let result = self.execute_query(query_name, &selection_set, arguments).await?;

                Content::json(result)?
            }
            _ => {
                // Handle unknown tool names or implement the query execution logic
                return Err(ErrorData::invalid_request(format!("Unknown tool name: {}", name), None));
                // TODO: Implement logic for query-* tools based on extracted name parts
            }
        };

        Ok(CallToolResult {
            content: vec![content],
            is_error: Some(false),
        })
    }
}

enum ToolType {
    Query,
    Mutation,
}

impl fmt::Display for ToolType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ToolType::Query => f.write_str("query"),
            ToolType::Mutation => f.write_str("mutation"),
        }
    }
}

fn add_field_to_tools(tool_type: ToolType, tools: &mut Vec<Tool>, field: engine_schema::FieldDefinition<'_>) {
    // Create root structure
    let mut properties = Map::new();

    let mut required_arguments = Vec::new();

    for argument in field.arguments() {
        if argument.ty().wrapping.is_required() {
            required_arguments.push(argument.name().to_string());
        }

        add_argument_to_properties(
            &mut properties,
            argument.ty().definition(),
            argument.name(),
            argument.description().unwrap_or_default(),
        );
    }

    let type_name = field.ty().definition().name();
    let wrapping = field.ty().wrapping;

    let type_description = if wrapping.is_list() {
        if wrapping.is_nullable() {
            if wrapping.inner_is_required() {
                "a nullable array of non-nullable items"
            } else {
                "a nullable array of nullable items"
            }
        } else if wrapping.inner_is_required() {
            "a non-nullable array of non-nullable items"
        } else {
            "a non-nullable array of nullable items"
        }
    } else if wrapping.is_nullable() {
        "a nullable item"
    } else {
        "a non-nullable item"
    };

    let r#type = match field.ty().definition() {
        Definition::Enum(_) => "enum",
        Definition::InputObject(_) => "input object",
        Definition::Interface(_) => "interface",
        Definition::Object(_) => "object",
        Definition::Scalar(_) => "scalar",
        Definition::Union(_) => "union",
    };

    let description = formatdoc! {r#"
        This value is written in the syntax of a GraphQL selection set. Example: '{{ id name }}'.

        Before generating this field, call the `introspect-type` tool with type name: {type_name}

        The `introspect-type` tool returns with a GraphQL introspection response format, and tells you
        if the return type is an object, a union or an interface.

        If it's an object, you have to select at least one field from the type.

        If it's a union, you can select any fields from any of the possible types. Remember to use fragment spreads,
        if needed.

        If it's an interface, you can select any fields from any of the possible types, or with fields
        from the interface itself. Remember to use fragment spreads, if needed.
    "#};

    // Add simple string selection parameter
    properties.insert(
        "__selection".to_string(),
        json!({
            "type": "string",
            "description": description,
        }),
    );

    let parameters = json!({
        "type": "object",
        "properties": properties,
        "required": ["__selection"]
    });

    let field_description = field.description().unwrap_or("");

    let description = formatdoc! {r#"
        {field_description}

        This {tool_type} returns a {type} named {type_name}. It is {type_description}.
        Provide a GraphQL selection set for the query (e.g., '{{ id name }}').

        You must determine the fields of the type by calling the `introspect-type` tool first in
        this MCP server. It will return the needed information for you to build the selection.

        Do NOT call this {tool_type} before running introspection and knowing exactly what fields you can select.
    "#};

    tools.push(Tool::new(
        field.name().to_string(),
        description.trim_start().to_string(),
        parameters.as_object().unwrap().clone(),
    ));
}

#[must_use]
fn create_input_properties(input: InputObjectDefinition<'_>) -> serde_json::Value {
    let mut properties = Map::new();
    let mut required = Vec::new();

    for field in input.input_fields() {
        if field.ty().wrapping.is_required() {
            required.push(field.name());
        }

        add_argument_to_properties(
            &mut properties,
            field.ty().definition(),
            field.name(),
            field.description().unwrap_or_default(),
        );
    }

    json!({
        "type": "object",
        "properties": properties,
        "required": required,
        "description": input.description().unwrap_or_default(),
    })
}

fn add_argument_to_properties(
    properties: &mut Map<String, Value>,
    definition: Definition<'_>,
    name: &str,
    description: &str,
) {
    match definition {
        Definition::Scalar(scalar) => {
            let r#type = match scalar.ty {
                ScalarType::String => "string",
                ScalarType::Float => "number",
                ScalarType::Int => "integer",
                ScalarType::BigInt => "integer",
                ScalarType::Boolean => "boolean",
                ScalarType::Unknown => "string",
            };

            properties.insert(
                name.to_string(),
                json!({
                    "type": r#type,
                    "description": description
                }),
            );
        }
        Definition::Enum(r#enum) => {
            let values = r#enum.values().map(|v| v.name()).collect::<Vec<_>>().join(", ");
            let description = format!("Must be one of: {values}. {description}");

            properties.insert(
                name.to_string(),
                json!({
                    "type": "string",
                    "description": description.strip_suffix(' '),
                }),
            );
        }
        Definition::InputObject(input) => {
            properties.insert(name.to_string(), create_input_properties(input));
        }
        Definition::Interface(_) => unreachable!(),
        Definition::Object(_) => unreachable!(),
        Definition::Union(_) => unreachable!(),
    }
}
