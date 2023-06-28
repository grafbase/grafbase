mod input_type;

use crate::{typescript_configuration, Document, GeneratorError, Result};
use async_graphql_parser::types::{TypeKind, TypeSystemDefinition};
use std::path::PathBuf;

pub fn generate(graphql_schema: impl AsRef<str>) -> Result<String> {
    let graphql_schema = async_graphql_parser::parse_schema(graphql_schema).map_err(GeneratorError::GraphQLParse)?;

    let mut document = Document::new();

    for definition in &graphql_schema.definitions {
        match definition {
            TypeSystemDefinition::Type(type_definition) => {
                let node = &type_definition.node;

                match &node.kind {
                    TypeKind::Scalar => (),
                    TypeKind::Object(_) => (),
                    TypeKind::Interface(_) => (),
                    TypeKind::Union(_) => (),
                    TypeKind::Enum(_) => (),
                    TypeKind::InputObject(ref obj) => {
                        let interface = input_type::generate(
                            node.name.node.as_str(),
                            node.description.as_ref().map(|description| description.node.as_str()),
                            obj,
                        );

                        document.push_item(interface);
                    }
                }
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
