use std::collections::HashMap;
use std::fmt;

use grafbase_sdk::{
    SelectionSetResolverExtension,
    types::{
        ArgumentValues, Configuration, Data, Error, Field, SubgraphHeaders, SubgraphSchema, TypeDefinition,
        WrappingType,
    },
};
use serde_json::{Value, json};

#[derive(SelectionSetResolverExtension)]
struct Resolver {
    config: Value,
    schemas: Value,
}

impl fmt::Debug for Resolver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Resolver")
            .field("config", &self.config)
            .field("schemas", &self.schemas)
            .finish()
    }
}

impl SelectionSetResolverExtension for Resolver {
    fn new(subgraph_schemas: Vec<SubgraphSchema<'_>>, config: Configuration) -> Result<Self, Error> {
        let config: Value = config.deserialize()?;

        // Serialize subgraph schemas into a JSON value
        let schemas = json!(
            subgraph_schemas
                .into_iter()
                .map(|subgraph_schema| {
                    let name = subgraph_schema.name();
                    let names = subgraph_schema
                        .type_definitions()
                        .map(|ty| (ty.id(), ty.name()))
                        .collect::<HashMap<_, _>>();

                    // Convert type definitions to JSON objects
                    let definitions = subgraph_schema
                        .type_definitions()
                        .map(|def| match def {
                            TypeDefinition::Scalar(scalar) => {
                                json!({
                                    scalar.name(): {
                                        "kind": "SCALAR",
                                        "specifiedByUrl": scalar.specified_by_url(),
                                        "directives": scalar.directives().map(|d| {
                                            json!({
                                                "name": d.name(), 
                                                "arguments": d.arguments::<Value>().unwrap_or_default() 
                                            })
                                        }).collect::<Vec<_>>()
                                    }
                                })
                            }
                            TypeDefinition::Object(obj) => {
                                json!({
                                    obj.name(): {
                                        "kind": "OBJECT",
                                        "fields": obj.fields().map(|f| {
                                            json!({
                                                "name": f.name(),
                                                "type": {
                                                    "definitionId": names.get(&f.ty().definition_id()),
                                                    "wrapping": f.ty().wrapping().map(|w| match w {
                                                        WrappingType::NonNull => "NON_NULL",
                                                        WrappingType::List => "LIST",
                                                    }).collect::<Vec<_>>()
                                                },
                                                "arguments": f.arguments().map(|arg| {
                                                        json!({
                                                            "name": arg.name(),
                                                            "type": {
                                                                "definitionId": names.get(&arg.ty().definition_id()),
                                                                "wrapping": arg.ty().wrapping().map(|w| match w {
                                                                    WrappingType::NonNull => "NON_NULL",
                                                                    WrappingType::List => "LIST",
                                                                }).collect::<Vec<_>>()
                                                            },
                                                            "directives": arg.directives().map(|d| {
                                                                json!({
                                                                    "name": d.name(), 
                                                                    "arguments": d.arguments::<Value>().unwrap_or_default() 
                                                                })
                                                            }).collect::<Vec<_>>()
                                                        })
                                                    }).collect::<Vec<_>>(),
                                                "directives": f.directives().map(|d| {
                                                    json!({
                                                        "name": d.name(), 
                                                        "arguments": d.arguments::<Value>().unwrap_or_default() 
                                                    })
                                                }).collect::<Vec<_>>()
                                            })
                                        }).collect::<Vec<_>>(),
                                        "interfaces": obj.interfaces().map(|id| names.get(&id)).collect::<Vec<_>>(),
                                        "directives": obj.directives().map(|d| {
                                            json!({
                                                "name": d.name(), 
                                                "arguments": d.arguments::<Value>().unwrap_or_default() 
                                            })
                                        }).collect::<Vec<_>>()
                                    }
                                })
                            }
                            TypeDefinition::Interface(interface) => {
                                json!({
                                    interface.name(): {
                                        "kind": "INTERFACE",
                                        "fields": interface.fields().map(|f| {
                                            json!({
                                                "name": f.name(),
                                                "type": {
                                                    "definitionId": names.get(&f.ty().definition_id()),
                                                    "wrapping": f.ty().wrapping().map(|w| match w {
                                                        WrappingType::NonNull => "NON_NULL",
                                                        WrappingType::List => "LIST",
                                                    }).collect::<Vec<_>>()
                                                },
                                                "arguments": f.arguments().map(|arg| {
                                                        json!({
                                                            "name": arg.name(),
                                                            "type": {
                                                                "definitionId": names.get(&arg.ty().definition_id()),
                                                                "wrapping": arg.ty().wrapping().map(|w| match w {
                                                                    WrappingType::NonNull => "NON_NULL",
                                                                    WrappingType::List => "LIST",
                                                                }).collect::<Vec<_>>()
                                                            },
                                                            "directives": arg.directives().map(|d| d.name()).collect::<Vec<_>>()
                                                        })
                                                    }).collect::<Vec<_>>(),
                                                "directives": f.directives().map(|d| {
                                                    json!({
                                                        "name": d.name(), 
                                                        "arguments": d.arguments::<Value>().unwrap_or_default() 
                                                    })
                                                }).collect::<Vec<_>>()
                                            })
                                        }).collect::<Vec<_>>(),
                                        "interfaces": interface.interfaces().map(|id| names.get(&id)).collect::<Vec<_>>(),
                                        "directives": interface.directives().map(|d| {
                                            json!({
                                                "name": d.name(), 
                                                "arguments": d.arguments::<Value>().unwrap_or_default() 
                                            })
                                        }).collect::<Vec<_>>()
                                    }
                                })
                            }
                            TypeDefinition::Union(union) => {
                                json!({
                                    union.name(): {
                                        "kind": "UNION",
                                        "memberTypes": union.member_types().map(|id| names.get(&id)).collect::<Vec<_>>(),
                                        "directives": union.directives().map(|d| {
                                            json!({
                                                "name": d.name(), 
                                                "arguments": d.arguments::<Value>().unwrap_or_default()
                                            })
                                        }).collect::<Vec<_>>()
                                    }
                                })
                            }
                            TypeDefinition::Enum(enum_def) => {
                                json!({
                                    enum_def.name(): {
                                        "kind": "ENUM",
                                        "name": enum_def.name(),
                                        "values": enum_def.values().map(|v| {
                                            json!({
                                                "name": v.name(),
                                                "directives": v.directives().map(|d| {
                                                    json!({
                                                        "name": d.name(), 
                                                        "arguments": d.arguments::<Value>().unwrap_or_default() 
                                                    })
                                                }).collect::<Vec<_>>()
                                            })
                                        }).collect::<Vec<_>>(),
                                        "directives": enum_def.directives().map(|d| {
                                            json!({
                                                "name": d.name(), 
                                                "arguments": d.arguments::<Value>().unwrap_or_default() 
                                            })
                                        }).collect::<Vec<_>>()
                                    }
                                })
                            }
                            TypeDefinition::InputObject(input) => {
                                json!({
                                    input.name(): {
                                        "kind": "INPUT_OBJECT",
                                        "inputFields": input.input_fields().map(|f| {
                                            json!({
                                                "name": f.name(),
                                                "type": {
                                                    "definitionId": names.get(&f.ty().definition_id()),
                                                    "wrapping": f.ty().wrapping().map(|w| match w {
                                                        WrappingType::NonNull => "NON_NULL",
                                                        WrappingType::List => "LIST",
                                                    }).collect::<Vec<_>>()
                                                },
                                                "directives": f.directives().map(|d| {
                                                    json!({
                                                        "name": d.name(), 
                                                        "arguments": d.arguments::<Value>().unwrap_or_default() 
                                                    })
                                                }).collect::<Vec<_>>()
                                            })
                                        }).collect::<Vec<_>>(),
                                        "directives": input.directives().map(|d| {
                                            json!({
                                                "name": d.name(), 
                                                "arguments": d.arguments::<Value>().unwrap_or_default() 
                                            })
                                        }).collect::<Vec<_>>()
                                    }
                                })
                            }
                        })
                        .collect::<Vec<_>>();

                    // Return a JSON object for this subgraph
                    json!({
                        "name": name,
                        "typeDefinitions": definitions,
                        "directives": subgraph_schema.directives().map(|d| {
                            json!({
                                "name": d.name(), 
                                "arguments": d.arguments::<Value>().unwrap_or_default() 
                            })
                        }).collect::<Vec<_>>()
                    })
                })
                .collect::<Vec<_>>()
        );
        Ok(Self { config, schemas })
    }

    fn prepare(&mut self, _subgraph_name: &str, field: Field<'_>) -> Result<Vec<u8>, Error> {
        Ok(field.into_bytes())
    }

    fn resolve(
        &mut self,
        _headers: SubgraphHeaders,
        subgraph_name: &str,
        prepared: &[u8],
        _arguments: ArgumentValues<'_>,
    ) -> Result<Data, Error> {
        // Sanity check for now.
        Field::with_bytes(prepared, |_| ())?;
        match subgraph_name {
            "echo-config" => Ok(Data::new(&self.config)?),
            "echo-schema" => Ok(Data::new(&self.schemas)?),
            _ => Err(Error::new(
                "Unknown subgraph. Only 'echo-config' and 'echo-schema' is supported.",
            )),
        }
    }
}
