use std::borrow::Cow;

use engine_schema::TypeDefinition;
use rmcp::model::{CallToolResult, Content};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{SearchTool, Tool};
use crate::EngineWatcher;

pub struct IntrospectTool<R: engine::Runtime> {
    engine: EngineWatcher<R>,
    enable_mutations: bool,
}

impl<R: engine::Runtime> Tool for IntrospectTool<R> {
    type Parameters = IntrospectionParameters;

    fn name() -> &'static str {
        "introspect"
    }

    fn description(&self) -> Cow<'_, str> {
        format!("Provides detailed information about GraphQL type definition. Always use `{}` first to identify relevant fields before if information on a specific type was not explicitly requested. If you're not certain whether a field exist on a type, always use this tool first.", SearchTool::<R>::name()).into()
    }

    async fn call(&self, parameters: Self::Parameters) -> anyhow::Result<CallToolResult> {
        let out = self.introspect(parameters.types);
        Ok(CallToolResult {
            content: vec![Content::json(out).unwrap()],
            is_error: None,
        })
    }
}

#[derive(Deserialize, JsonSchema)]
pub struct IntrospectionParameters {
    types: Vec<String>,
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum TypeOrError {
    Type(Type),
    Error(String),
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Type {
    pub name: String,
    pub kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<Field>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_fields: Option<Vec<InputValue>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<EnumValue>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub possible_types: Option<Vec<String>>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub is_deprecated: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecation_reason: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Field {
    pub name: String,
    pub r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<InputValue>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub is_deprecated: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecation_reason: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InputValue {
    pub name: String,
    pub r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_value: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct EnumValue {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub is_deprecated: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecation_reason: Option<String>,
}

impl<'a> From<engine_schema::FieldDefinition<'a>> for Field {
    fn from(field: engine_schema::FieldDefinition<'a>) -> Self {
        let (is_deprecated, deprecation_reason) = if let Some(deprecated) = field.has_deprecated() {
            (true, deprecated.reason().map(str::to_owned))
        } else {
            (false, None)
        };

        Self {
            name: field.name().to_owned(),
            r#type: field.ty().to_string(),
            description: field.description().map(str::to_owned),
            args: field.arguments().map(Into::into).collect(),
            is_deprecated,
            deprecation_reason,
        }
    }
}

impl<'a> From<engine_schema::TypeDefinition<'a>> for Type {
    fn from(type_definition: engine_schema::TypeDefinition<'a>) -> Self {
        let mut type_info = Type {
            name: type_definition.name().to_owned(),
            kind: match type_definition {
                TypeDefinition::Scalar(_) => "SCALAR",
                TypeDefinition::Object(_) => "OBJECT",
                TypeDefinition::Interface(_) => "INTERFACE",
                TypeDefinition::Union(_) => "UNION",
                TypeDefinition::Enum(_) => "ENUM",
                TypeDefinition::InputObject(_) => "INPUT_OBJECT",
            },
            description: type_definition.description().map(str::to_owned),
            fields: None,
            input_fields: None,
            possible_types: None,
            is_deprecated: false,
            deprecation_reason: None,
            enum_values: None,
        };

        if let Some(deprecated) = type_definition.has_deprecated() {
            type_info.is_deprecated = true;
            type_info.deprecation_reason = deprecated.reason().map(str::to_owned);
        }

        match type_definition {
            TypeDefinition::Object(obj) => {
                type_info.fields = Some(obj.fields().map(Into::into).collect());
            }
            TypeDefinition::Interface(inf) => {
                type_info.fields = Some(inf.fields().map(Into::into).collect());
                type_info.possible_types = Some(inf.possible_types().map(|t| t.name().to_owned()).collect());
            }
            TypeDefinition::Union(union) => {
                type_info.possible_types = Some(union.possible_types().map(|t| t.name().to_owned()).collect());
            }
            TypeDefinition::InputObject(obj) => {
                type_info.input_fields = Some(obj.input_fields().map(Into::into).collect())
            }
            TypeDefinition::Enum(enm) => {
                type_info.enum_values = Some(enm.values().map(Into::into).collect());
            }
            TypeDefinition::Scalar(_) => {}
        }

        type_info
    }
}

impl<'a> From<engine_schema::InputValueDefinition<'a>> for InputValue {
    fn from(input: engine_schema::InputValueDefinition<'a>) -> Self {
        Self {
            name: input.name().to_owned(),
            r#type: input.ty().to_string(),
            description: input.description().map(str::to_owned),
            default_value: input.default_value().map(|v| serde_json::to_value(v).unwrap()),
        }
    }
}

impl<'a> From<engine_schema::EnumValue<'a>> for EnumValue {
    fn from(enum_value: engine_schema::EnumValue<'a>) -> Self {
        let (is_deprecated, deprecation_reason) = if let Some(deprecated) = enum_value.has_deprecated() {
            (true, deprecated.reason().map(str::to_owned))
        } else {
            (false, None)
        };

        Self {
            name: enum_value.name().to_owned(),
            description: enum_value.description().map(str::to_owned),
            is_deprecated,
            deprecation_reason,
        }
    }
}

impl<R: engine::Runtime> IntrospectTool<R> {
    pub fn new(engine: &EngineWatcher<R>, enable_mutations: bool) -> Self {
        Self {
            engine: engine.clone(),
            enable_mutations,
        }
    }

    fn introspect(&self, types: Vec<String>) -> Vec<TypeOrError> {
        let schema = self.engine.borrow().schema.clone();
        let mut out = Vec::new();

        for type_name in types {
            let Some(type_definition) = schema.type_definition_by_name(&type_name) else {
                out.push(TypeOrError::Error(format!("Type '{}' not found", type_name)));
                continue;
            };

            if !self.enable_mutations
                && schema
                    .mutation()
                    .zip(type_definition.id().as_object())
                    .map(|(mutation, id)| mutation.id == id)
                    .unwrap_or_default()
            {
                out.push(TypeOrError::Error(format!("Type '{}' not found", type_name)));
                continue;
            }

            out.push(TypeOrError::Type(type_definition.into()));
        }

        out
    }
}
