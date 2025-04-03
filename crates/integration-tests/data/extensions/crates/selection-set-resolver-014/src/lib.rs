use std::collections::HashMap;
use std::fmt;

use grafbase_sdk::{
    SelectionSetResolverExtension,
    types::{
        ArgumentValues, Configuration, Data, DefinitionId, Error, Field, SubgraphHeaders, SubgraphSchema,
        TypeDefinition, WrappingType,
    },
};
use serde_json::{Value, json};

#[derive(SelectionSetResolverExtension)]
struct Resolver {
    config: Value,
    schemas: Value,
    definitions_by_subgraph_name: Vec<(String, HashMap<DefinitionId, String>)>,
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
        let mut schemas = Vec::new();
        let mut definitions_by_subgraph_name = Vec::new();

        for subgraph_schema in subgraph_schemas {
            let subgraph_name = subgraph_schema.name();
            let mut names = subgraph_schema
                .type_definitions()
                .map(|ty| (ty.id(), ty.name().to_owned()))
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
                        for field in obj.fields() {
                            names.insert(field.id(), format!("{}.{}", obj.name(), field.name()));
                        }
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
                        for field in interface.fields() {
                            names.insert(field.id(), format!("{}.{}", interface.name(), field.name()));
                        }
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
            schemas.push(json!({
                "name": subgraph_name,
                "typeDefinitions": definitions,
                "directives": subgraph_schema.directives().map(|d| {
                    json!({
                        "name": d.name(),
                        "arguments": d.arguments::<Value>().unwrap_or_default()
                    })
                }).collect::<Vec<_>>()
            }));
            definitions_by_subgraph_name.push((subgraph_name.to_owned(), names));
        }
        Ok(Self {
            config,
            schemas: Value::Array(schemas),
            definitions_by_subgraph_name,
        })
    }

    fn prepare(&mut self, _subgraph_name: &str, field: Field<'_>) -> Result<Vec<u8>, Error> {
        Ok(field.into_bytes())
    }

    fn resolve(
        &mut self,
        _headers: SubgraphHeaders,
        subgraph_name: &str,
        prepared: &[u8],
        arguments: ArgumentValues<'_>,
    ) -> Result<Data, Error> {
        let names = self
            .definitions_by_subgraph_name
            .iter()
            .find_map(|(name, names)| if name == subgraph_name { Some(names) } else { None })
            .unwrap();

        struct Ctx<'a> {
            names: &'a HashMap<DefinitionId, String>,
            arguments: ArgumentValues<'a>,
        }

        impl Ctx<'_> {
            fn process_field(&self, field: Field<'_>) -> Value {
                let mut field_json = json!({
                    "id": self.names.get(&field.definition_id()),
                });

                if let Ok(value) = field.arguments::<Value>(self.arguments) {
                    field_json["arguments"] = value;
                }

                if let Some(alias) = field.alias() {
                    field_json["alias"] = alias.into();
                }

                // Process selection set if it exists
                if let Some(selection_set) = field.selection_set() {
                    let fields = selection_set
                        .fields_ordered_by_parent_entity()
                        .map(|field| self.process_field(field))
                        .collect::<Vec<_>>();

                    field_json["selectionSet"] = json!({
                        "fields": fields,
                        "requiresTypename": selection_set.requires_typename(),
                    });
                }

                field_json
            }
        }

        // Create a detailed representation of the selection set
        let selection_set = Field::with_bytes(prepared, |field| {
            // Start processing from the root field
            Ctx { names, arguments }.process_field(field)
        })?;

        match subgraph_name {
            "echo-config" => Ok(Data::new(&self.config)?),
            "echo-schema" => Ok(Data::new(&self.schemas)?),
            "echo-selection-set" => Err(Error::new("MyError").with_extension("selectionSet", selection_set)?),
            _ => Err(Error::new(format!("Unknown subgraph '{subgraph_name}"))),
        }
    }
}
