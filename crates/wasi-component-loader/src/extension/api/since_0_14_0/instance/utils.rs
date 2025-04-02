use engine_schema::{Schema, SubgraphId, TypeDefinition, TypeSystemDirective};
use extension_catalog::ExtensionId;

use crate::{cbor, extension::api::since_0_14_0::wit::schema as ws};

pub fn create_subgraph_schema_directives(schema: &Schema, extension_id: ExtensionId) -> Vec<(&str, ws::Schema<'_>)> {
    let mut subgraph_schemas = Vec::new();
    for subgraph in schema.subgraphs() {
        let mut directives = Vec::new();

        for schema_directive in subgraph.extension_schema_directives() {
            if schema_directive.extension_id != extension_id {
                continue;
            }

            directives
                .push(transform_directive(TypeSystemDirective::Extension(schema_directive), extension_id).unwrap());
        }

        if !directives.is_empty() {
            subgraph_schemas.push((
                subgraph.name(),
                ws::Schema {
                    directives,
                    definitions: Vec::new(),
                },
            ));
        }
    }
    subgraph_schemas
}

pub fn create_complete_subgraph_schemas(schema: &Schema, extension_id: ExtensionId) -> Vec<(&str, ws::Schema<'_>)> {
    let subgraph_ids = {
        let mut ids = schema
            .resolvers()
            .filter_map(|resolver| match resolver.variant() {
                engine_schema::ResolverDefinitionVariant::SelectionSetResolverExtension(res)
                    if res.extension_id == extension_id =>
                {
                    Some(res.subgraph_id)
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        ids.sort_unstable();
        ids.dedup();
        ids.into_iter().map(SubgraphId::Virtual).collect::<Vec<_>>()
    };
    let mut subgraph_schemas = subgraph_ids
        .into_iter()
        .map(|id| {
            (
                id,
                ws::Schema {
                    directives: Vec::new(),
                    definitions: Vec::new(),
                },
            )
        })
        .collect::<Vec<_>>();

    for definition in schema.type_definitions() {
        match definition {
            TypeDefinition::Enum(enum_def) => {
                for (id, subgraph_schema) in &mut subgraph_schemas {
                    if enum_def.exists_in_subgraph_ids.contains(id) {
                        let def = ws::Definition::Enum(ws::EnumDefinition {
                            id: enum_def.id.as_guid(),
                            name: enum_def.name(),
                            values: enum_def
                                .values()
                                .map(|value| ws::EnumValue {
                                    name: value.name(),
                                    directives: collect_extension_directives(value.directives(), extension_id),
                                })
                                .collect(),
                            directives: collect_extension_directives(enum_def.directives(), extension_id),
                        });
                        subgraph_schema.definitions.push(def);
                    }
                }
            }
            TypeDefinition::InputObject(input_obj) => {
                for (id, subgraph_schema) in &mut subgraph_schemas {
                    if input_obj.exists_in_subgraph_ids.contains(id) {
                        let def = ws::Definition::InputObject(ws::InputObjectDefinition {
                            id: input_obj.id.as_guid(),
                            name: input_obj.name(),
                            input_fields: input_obj
                                .input_fields()
                                .map(|field| ws::InputValueDefinition {
                                    id: field.id.as_guid(),
                                    name: field.name(),
                                    ty: field.ty().into(),
                                    directives: collect_extension_directives(field.directives(), extension_id),
                                })
                                .collect(),
                            directives: collect_extension_directives(input_obj.directives(), extension_id),
                        });
                        subgraph_schema.definitions.push(def);
                    }
                }
            }
            TypeDefinition::Interface(interface) => {
                for (id, subgraph_schema) in &mut subgraph_schemas {
                    if interface.exists_in_subgraph_ids.contains(id) {
                        let def = ws::Definition::Interface(ws::InterfaceDefinition {
                            id: interface.id.as_guid(),
                            name: interface.name(),
                            interfaces: interface.interfaces().map(|inf| inf.id.as_guid()).collect(),
                            fields: interface
                                .fields()
                                .map(|field| ws::FieldDefinition {
                                    id: field.id.as_guid(),
                                    name: field.name(),
                                    arguments: field
                                        .arguments()
                                        .map(|arg| ws::InputValueDefinition {
                                            id: arg.id.as_guid(),
                                            name: arg.name(),
                                            ty: arg.ty().into(),
                                            directives: collect_extension_directives(arg.directives(), extension_id),
                                        })
                                        .collect(),
                                    ty: field.ty().into(),
                                    directives: collect_extension_directives(field.directives(), extension_id),
                                })
                                .collect(),
                            directives: collect_extension_directives(interface.directives(), extension_id),
                        });
                        subgraph_schema.definitions.push(def);
                    }
                }
            }
            TypeDefinition::Object(obj) => {
                for (id, subgraph_schema) in &mut subgraph_schemas {
                    if obj.exists_in_subgraph_ids.contains(id) {
                        let def = ws::Definition::Object(ws::ObjectDefinition {
                            id: obj.id.as_guid(),
                            name: obj.name(),
                            interfaces: obj.interfaces().map(|inf| inf.id.as_guid()).collect(),
                            fields: obj
                                .fields()
                                .map(|field| ws::FieldDefinition {
                                    id: field.id.as_guid(),
                                    name: field.name(),
                                    arguments: field
                                        .arguments()
                                        .map(|arg| ws::InputValueDefinition {
                                            id: arg.id.as_guid(),
                                            name: arg.name(),
                                            ty: arg.ty().into(),
                                            directives: collect_extension_directives(arg.directives(), extension_id),
                                        })
                                        .collect(),
                                    ty: field.ty().into(),
                                    directives: collect_extension_directives(field.directives(), extension_id),
                                })
                                .collect(),
                            directives: collect_extension_directives(obj.directives(), extension_id),
                        });
                        subgraph_schema.definitions.push(def);
                    }
                }
            }
            TypeDefinition::Scalar(scalar) => {
                for (id, subgraph_schema) in &mut subgraph_schemas {
                    if scalar.exists_in_subgraph_ids.contains(id) {
                        let def = ws::Definition::Scalar(ws::ScalarDefinition {
                            id: scalar.id.as_guid(),
                            name: scalar.name(),
                            specified_by_url: scalar.specified_by_url(),
                            directives: collect_extension_directives(scalar.directives(), extension_id),
                        });
                        subgraph_schema.definitions.push(def);
                    }
                }
            }
            TypeDefinition::Union(union_def) => {
                for (id, subgraph_schema) in &mut subgraph_schemas {
                    if union_def.exists_in_subgraph_ids.contains(id) {
                        let def = ws::Definition::Union(ws::UnionDefinition {
                            id: union_def.id.as_guid(),
                            name: union_def.name(),
                            member_types: union_def.possible_types().map(|obj| obj.id.as_guid()).collect(),
                            directives: collect_extension_directives(union_def.directives(), extension_id),
                        });
                        subgraph_schema.definitions.push(def);
                    }
                }
            }
        }
    }

    subgraph_schemas
        .into_iter()
        .map(|(id, subgraph_schema)| (schema.walk(id).name(), subgraph_schema))
        .collect::<Vec<(&str, ws::Schema<'_>)>>()
}

fn transform_directive(directive: TypeSystemDirective<'_>, extension_id: ExtensionId) -> Option<ws::Directive<'_>> {
    match directive {
        TypeSystemDirective::Extension(dir) if dir.extension_id == extension_id => Some(ws::Directive {
            name: dir.name(),
            arguments: cbor::to_vec(dir.static_arguments()).unwrap(),
        }),
        _ => None,
    }
}

fn collect_extension_directives<'a, I>(directives: I, extension_id: ExtensionId) -> Vec<ws::Directive<'a>>
where
    I: Iterator<Item = TypeSystemDirective<'a>>,
{
    directives
        .filter_map(|dir| transform_directive(dir, extension_id))
        .collect()
}
