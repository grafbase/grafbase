mod input_type;
mod r#type;

use crate::{document::DocumentItem, r#type::StaticType, typescript_configuration, Document, GeneratorError, Result};
use async_graphql_parser::types::{BaseType, TypeKind, TypeSystemDefinition};
use std::path::PathBuf;

/// Transpiles a GraphQL schema definition to TypeScript client schema.
pub fn generate(graphql_schema: impl AsRef<str>) -> Result<String> {
    let graphql_schema = async_graphql_parser::parse_schema(graphql_schema).map_err(GeneratorError::GraphQLParse)?;

    let mut document = Document::new();

    for definition in &graphql_schema.definitions {
        match definition {
            TypeSystemDefinition::Type(type_definition) => {
                use TypeKind::*;

                let node = &type_definition.node;
                let name = node.name.node.as_str();
                let description = node.description.as_ref().map(|description| description.node.as_str());

                let item: DocumentItem<'_> = match &node.kind {
                    Scalar => todo!(),
                    Object(ref obj) => r#type::generate(name, description, obj).into(),
                    Interface(_) => todo!(),
                    Union(_) => todo!(),
                    Enum(_) => todo!(),
                    InputObject(ref obj) => input_type::generate(name, description, obj).into(),
                };

                document.push_item(item);
            }
            TypeSystemDefinition::Directive(_) => (),
            TypeSystemDefinition::Schema(_) => (),
        }
    }

    let result = document.to_string();
    let result = dprint_plugin_typescript::format_text(&PathBuf::from("test.ts"), &result, typescript_configuration())
        .map_err(|e| GeneratorError::TypeScriptGenerate(e.to_string()))?
        .unwrap_or(result);

    Ok(result)
}

pub(super) fn generate_base_type<'a>(base: &'a BaseType, nullable: bool) -> StaticType<'a> {
    static STRING_TYPES: &[&str] = &["String", "ID", "Email", "IPAddress", "URL", "PhoneNumber"];
    static DATE_TYPES: &[&str] = &["Date", "DateTime", "Timestamp"];
    static NUMBER_TYPES: &[&str] = &["Int", "Float"];

    match base {
        BaseType::Named(ref name) => {
            let mut r#type = match name.as_str() {
                name if STRING_TYPES.contains(&name) => StaticType::ident("string"),
                name if DATE_TYPES.contains(&name) => StaticType::ident("Date"),
                name if NUMBER_TYPES.contains(&name) => StaticType::ident("number"),
                "Boolean" => StaticType::ident("boolean"),
                "JSON" => StaticType::ident("object"),
                name => StaticType::ident(name),
            };

            if nullable {
                r#type.or(StaticType::null());
            }

            r#type
        }
        BaseType::List(ref base) => {
            let mut r#type = dbg!(generate_base_type(&base.base, base.nullable));

            match (nullable, base.nullable) {
                (true, true) => {
                    r#type.array();
                    r#type = StaticType::new(r#type);
                    r#type.or(StaticType::null())
                }
                (true, false) => {
                    r#type.array();
                    r#type = StaticType::new(r#type);
                    r#type.or(StaticType::null());
                }
                (false, true) => {
                    r#type = StaticType::new(r#type);
                    r#type.array();
                }
                (false, false) => {
                    r#type = StaticType::new(r#type);
                    r#type.array();
                }
            }

            r#type
        }
    }
}
