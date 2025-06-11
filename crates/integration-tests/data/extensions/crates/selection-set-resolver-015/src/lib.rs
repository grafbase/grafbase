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
    field_names_by_subgraph_name: Vec<(String, HashMap<DefinitionId, String>)>,
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
        if let Some(msg) = config.get("error").and_then(|v| v.as_str()) {
            return Err(Error::new(msg.to_string()));
        }

        let mut schemas = Vec::new();
        let mut definitions_by_subgraph_name = Vec::new();

        for subgraph_schema in subgraph_schemas {
            let subgraph_name = subgraph_schema.name();
            let mut field_names = HashMap::new();

            // Convert type definitions to JSON objects
            let mut type_definitions = subgraph_schema
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
                            field_names.insert(field.id(), format!("{}.{}", obj.name(), field.name()));
                        }
                        json!({
                            obj.name(): {
                                "kind": "OBJECT",
                                "fields": obj.fields().map(|f| {
                                    json!({
                                        "name": f.name(),
                                        "type": {
                                            "definition": &f.ty().definition().name(),
                                            "wrapping": f.ty().wrapping().map(|w| match w {
                                                WrappingType::NonNull => "NON_NULL",
                                                WrappingType::List => "LIST",
                                            }).collect::<Vec<_>>()
                                        },
                                        "arguments": f.arguments().map(|arg| {
                                                json!({
                                                    "name": arg.name(),
                                                    "type": {
                                                        "definition": arg.ty().definition().name(),
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
                                "interfaces": obj.interfaces().map(|inf| inf.name()).collect::<Vec<_>>(),
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
                            field_names.insert(field.id(), format!("{}.{}", interface.name(), field.name()));
                        }
                        json!({
                            interface.name(): {
                                "kind": "INTERFACE",
                                "fields": interface.fields().map(|f| {
                                    json!({
                                        "name": f.name(),
                                        "type": {
                                            "definition":f.ty().definition().name(),
                                            "wrapping": f.ty().wrapping().map(|w| match w {
                                                WrappingType::NonNull => "NON_NULL",
                                                WrappingType::List => "LIST",
                                            }).collect::<Vec<_>>()
                                        },
                                        "arguments": f.arguments().map(|arg| {
                                                json!({
                                                    "name": arg.name(),
                                                    "type": {
                                                        "definition": arg.ty().definition().name(),
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
                                "interfaces": interface.interfaces().map(|inf| inf.name()).collect::<Vec<_>>(),
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
                                "memberTypes": union.member_types().map(|obj| obj.name()).collect::<Vec<_>>(),
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
                                            "definition": f.ty().definition().name(),
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

            type_definitions.sort_by(|a, b| {
                let a = a
                    .as_object()
                    .and_then(|obj| obj.keys().next())
                    .map(String::as_str)
                    .unwrap_or("");
                let b = b
                    .as_object()
                    .and_then(|obj| obj.keys().next())
                    .map(String::as_str)
                    .unwrap_or("");
                a.cmp(b)
            });

            let query = subgraph_schema.query().map(|obj| obj.name());
            let mutation = subgraph_schema.mutation().map(|obj| obj.name());
            let subscription = subgraph_schema.subscription().map(|obj| obj.name());

            // Return a JSON object for this subgraph
            schemas.push(json!({
                "name": subgraph_name,
                "query": query,
                "mutation": mutation,
                "subscription": subscription,
                "typeDefinitions": type_definitions,
                "directives": subgraph_schema.directives().map(|d| {
                    json!({
                        "name": d.name(),
                        "arguments": d.arguments::<Value>().unwrap_or_default()
                    })
                }).collect::<Vec<_>>()
            }));
            definitions_by_subgraph_name.push((subgraph_name.to_owned(), field_names));
        }
        Ok(Self {
            config,
            schemas: Value::Array(schemas),
            field_names_by_subgraph_name: definitions_by_subgraph_name,
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
        let field_names = self
            .field_names_by_subgraph_name
            .iter()
            .find_map(|(name, field_names)| if name == subgraph_name { Some(field_names) } else { None })
            .ok_or_else(|| Error::new(format!("Unknown subgraph '{subgraph_name}'")))?;

        struct Ctx<'a> {
            field_names: &'a HashMap<DefinitionId, String>,
            arguments: ArgumentValues<'a>,
        }

        impl Ctx<'_> {
            fn process_field(&self, field: Field<'_>) -> Value {
                let mut field_json = json!({
                    "id": self.field_names.get(&field.definition_id()),
                });

                field_json["arguments"] = match field.arguments::<Value>(self.arguments) {
                    Ok(value) => value,
                    Err(err) => format!("ERROR: {err}").into(),
                };

                if let Some(alias) = field.alias() {
                    field_json["alias"] = alias.into();
                }

                // Process selection set if it exists
                let fields = field
                    .selection_set()
                    .fields_ordered_by_parent_entity()
                    .map(|field| self.process_field(field))
                    .collect::<Vec<_>>();

                if !fields.is_empty() {
                    field_json["selectionSet"] = json!({
                        "fields": fields,
                        "requiresTypename": field.selection_set().requires_typename(),
                    });
                }

                field_json
            }
        }

        // Create a detailed representation of the selection set
        let selection_set = Field::with_bytes(prepared, |field| {
            // Start processing from the root field
            Ctx { field_names, arguments }.process_field(field)
        })?;

        match subgraph_name {
            "echo-config" => Ok(Data::new(&self.config)?),
            "echo-schema" => Ok(Data::new(&self.schemas)?),
            "echo-selection-set" => Err(Error::new("MyError").with_extension("selectionSet", selection_set)?),
            _ => Err(Error::new(format!("Unknown subgraph '{subgraph_name}"))),
        }
    }
}
