use crate::{federated_graph::*, FederatedGraph};
use async_graphql_parser::{types as ast, Positioned};
use indexmap::IndexSet;
use std::{collections::HashMap, error::Error as StdError, fmt};

const JOIN_GRAPH_ENUM_NAME: &str = "join__Graph";
const JOIN_GRAPH_DIRECTIVE_NAME: &str = "join__graph";
const JOIN_FIELD_DIRECTIVE_NAME: &str = "join__field";
const JOIN_TYPE_DIRECTIVE_NAME: &str = "join__type";

#[derive(Debug)]
pub struct DomainError(String);

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl StdError for DomainError {}

#[derive(Default)]
struct State<'a> {
    subgraphs: Vec<Subgraph>,

    objects: Vec<Object>,
    object_fields: Vec<ObjectField>,

    interfaces: Vec<Interface>,
    interface_fields: Vec<InterfaceField>,

    fields: Vec<Field>,

    enums: Vec<Enum>,
    unions: Vec<Union>,
    scalars: Vec<Scalar>,
    input_objects: Vec<InputObject>,

    strings: IndexSet<String>,
    field_types: indexmap::IndexSet<FieldType>,

    query_type: Option<ObjectId>,
    mutation_type: Option<ObjectId>,
    subscription_type: Option<ObjectId>,

    definition_names: HashMap<&'a str, Definition>,
    selection_map: HashMap<(Definition, &'a str), FieldId>,

    /// The key is the name of the graph in the join__Graph enum.
    graph_sdl_names: HashMap<&'a str, SubgraphId>,
}

impl<'a> State<'a> {
    fn insert_field_type(&mut self, field_type: &'a ast::Type) -> FieldTypeId {
        let mut list_wrappers = Vec::new();
        let mut ty = field_type;

        let kind = loop {
            match &ty.base {
                ast::BaseType::Named(name) => break self.definition_names[name.as_str()],
                ast::BaseType::List(inner) => {
                    list_wrappers.push(if ty.nullable {
                        ListWrapper::NullableList
                    } else {
                        ListWrapper::RequiredList
                    });
                    ty = inner.as_ref();
                }
            }
        };

        let idx = self
            .field_types
            .insert_full(FieldType {
                kind,
                inner_is_required: !ty.nullable,
                list_wrappers,
            })
            .0;
        FieldTypeId(idx)
    }

    fn insert_string(&mut self, s: &str) -> StringId {
        if let Some(idx) = self.strings.get_index_of(s) {
            return StringId(idx);
        }

        StringId(self.strings.insert_full(s.to_owned()).0)
    }

    fn insert_value(&mut self, node: &async_graphql_value::ConstValue) -> Value {
        match node {
            async_graphql_value::ConstValue::Null => Value::String(self.insert_string("null")),
            async_graphql_value::ConstValue::Number(number) => Value::String(self.insert_string(&number.to_string())),
            async_graphql_value::ConstValue::String(s) => Value::String(self.insert_string(s)),
            async_graphql_value::ConstValue::Boolean(b) => Value::Boolean(*b),
            async_graphql_value::ConstValue::Enum(enm) => Value::EnumValue(self.insert_string(enm)),
            async_graphql_value::ConstValue::Binary(_) => unreachable!(),
            async_graphql_value::ConstValue::List(_) => todo!(),
            async_graphql_value::ConstValue::Object(_) => todo!(),
        }
    }
}

pub fn from_sdl(sdl: &str) -> Result<FederatedGraph, DomainError> {
    let mut state = State::default();
    let parsed = async_graphql_parser::parse_schema(sdl).map_err(|err| DomainError(err.to_string()))?;

    ingest_definitions(&parsed, &mut state)?;
    ingest_fields(&parsed, &mut state)?;
    // This needs to happen after all fields have been ingested, in order to attach selection sets.
    ingest_selection_sets(&parsed, &mut state)?;

    Ok(FederatedGraph::V1(FederatedGraphV1 {
        subgraphs: state.subgraphs,
        root_operation_types: RootOperationTypes {
            query: state
                .query_type
                .ok_or_else(|| DomainError("The `Query` type is not defined".to_owned()))?,
            mutation: state.mutation_type,
            subscription: state.subscription_type,
        },
        objects: state.objects,
        object_fields: state.object_fields,
        interfaces: state.interfaces,
        interface_fields: state.interface_fields,
        fields: state.fields,
        enums: state.enums,
        unions: state.unions,
        scalars: state.scalars,
        input_objects: state.input_objects,
        strings: state.strings.into_iter().collect(),
        field_types: state.field_types.into_iter().collect(),
    }))
}

fn ingest_fields<'a>(parsed: &'a ast::ServiceDocument, state: &mut State<'a>) -> Result<(), DomainError> {
    for definition in &parsed.definitions {
        match definition {
            ast::TypeSystemDefinition::Schema(_) => {
                return Err(DomainError(
                    "Not implemented: schema definitions in federated schema".to_owned(),
                ))
            }
            ast::TypeSystemDefinition::Directive(_) => (),
            ast::TypeSystemDefinition::Type(typedef) => match &typedef.node.kind {
                ast::TypeKind::Scalar => (),
                ast::TypeKind::Object(object) => {
                    let Definition::Object(object_id) = state.definition_names[typedef.node.name.node.as_str()] else {
                        return Err(DomainError(
                            "Broken invariant: object id behind object name.".to_owned(),
                        ));
                    };
                    ingest_object_interfaces(object_id, object, state)?;
                    ingest_object_fields(object_id, object, state);
                }
                ast::TypeKind::Interface(iface) => {
                    let Definition::Interface(interface_id) = state.definition_names[typedef.node.name.node.as_str()]
                    else {
                        return Err(DomainError(
                            "Broken invariant: interface id behind interface name.".to_owned(),
                        ));
                    };
                    ingest_interface_interfaces(interface_id, iface, state)?;
                    ingest_interface(interface_id, iface, state);
                }
                ast::TypeKind::Union(union) => {
                    let Definition::Union(union_id) = state.definition_names[typedef.node.name.node.as_str()] else {
                        return Err(DomainError("Broken invariant: UnionId behind union name.".to_owned()));
                    };
                    ingest_union_members(union_id, union, state)?;
                }
                ast::TypeKind::Enum(_) => {}
                ast::TypeKind::InputObject(input_object) => {
                    let Definition::InputObject(input_object_id) =
                        state.definition_names[typedef.node.name.node.as_str()]
                    else {
                        return Err(DomainError(
                            "Broken invariant: InputObjectId behind input object name.".to_owned(),
                        ));
                    };
                    ingest_input_object(input_object_id, input_object, state);
                }
            },
        }
    }

    Ok(())
}

fn ingest_interface_interfaces(
    interface_id: InterfaceId,
    interface: &ast::InterfaceType,
    state: &mut State<'_>,
) -> Result<(), DomainError> {
    state.interfaces[interface_id.0].implements_interfaces = interface
        .implements
        .iter()
        .map(|name| match state.definition_names[name.node.as_str()] {
            Definition::Interface(interface_id) => Ok(interface_id),
            _ => Err(DomainError(
                "Broken invariant: object implements non-interface type".to_owned(),
            )),
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(())
}

fn ingest_object_interfaces(
    object_id: ObjectId,
    object: &ast::ObjectType,
    state: &mut State<'_>,
) -> Result<(), DomainError> {
    state.objects[object_id.0].implements_interfaces = object
        .implements
        .iter()
        .map(|name| match state.definition_names[name.node.as_str()] {
            Definition::Interface(interface_id) => Ok(interface_id),
            _ => Err(DomainError(
                "Broken invariant: object implements non-interface type".to_owned(),
            )),
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(())
}

fn ingest_selection_sets(parsed: &ast::ServiceDocument, state: &mut State<'_>) -> Result<(), DomainError> {
    ingest_provides_requires(parsed, state)?;
    ingest_entity_keys(parsed, state)
}

fn ingest_entity_keys(parsed: &ast::ServiceDocument, state: &mut State<'_>) -> Result<(), DomainError> {
    for typedef in parsed.definitions.iter().filter_map(|def| match def {
        ast::TypeSystemDefinition::Type(ty) => Some(&ty.node),
        _ => None,
    }) {
        let Some(definition) = state.definition_names.get(typedef.name.node.as_str()).copied() else {
            continue;
        };
        for join_type in typedef
            .directives
            .iter()
            .filter(|dir| dir.node.name.node == JOIN_TYPE_DIRECTIVE_NAME)
        {
            let subgraph_id = join_type
                .node
                .get_argument("graph")
                .and_then(|arg| match &arg.node {
                    async_graphql_value::ConstValue::Enum(s) => Some(state.graph_sdl_names[s.as_str()]),
                    _ => None,
                })
                .expect("Missing graph argument in @join__type");
            let fields = join_type
                .node
                .get_argument("key")
                .and_then(|arg| match &arg.node {
                    async_graphql_value::ConstValue::String(s) => Some(s),
                    _ => None,
                })
                .map(|fields| {
                    parse_selection_set(fields).and_then(|fields| attach_selection(&fields, definition, state))
                })
                .transpose()?
                .unwrap_or_default();

            let is_interface_object = join_type
                .node
                .get_argument("isInterfaceObject")
                .map(|arg| matches!(arg.node, async_graphql_value::ConstValue::Boolean(true)))
                .unwrap_or(false);

            match definition {
                Definition::Object(object_id) => {
                    state.objects[object_id.0].resolvable_keys.push(Key {
                        subgraph_id,
                        fields,
                        is_interface_object,
                    });
                }
                Definition::Interface(interface_id) => {
                    state.interfaces[interface_id.0].resolvable_keys.push(Key {
                        subgraph_id,
                        fields,
                        is_interface_object,
                    });
                }
                _ => (),
            }
        }
    }

    Ok(())
}

fn ingest_provides_requires(parsed: &ast::ServiceDocument, state: &mut State<'_>) -> Result<(), DomainError> {
    let all_fields = parsed.definitions.iter().filter_map(|definition| match definition {
        ast::TypeSystemDefinition::Type(typedef) => {
            let type_name = typedef.node.name.node.as_str();
            match &typedef.node.kind {
                ast::TypeKind::Object(object) => Some((type_name, &object.fields)),
                ast::TypeKind::Interface(iface) => Some((type_name, &iface.fields)),
                _ => None,
            }
        }
        _ => None,
    });

    for (parent_name, field) in
        all_fields.flat_map(|(parent_name, fields)| fields.iter().map(move |field| (parent_name, &field.node)))
    {
        let Some(join_field_directive) = field
            .directives
            .iter()
            .find(|dir| dir.node.name.node == JOIN_FIELD_DIRECTIVE_NAME)
        else {
            continue;
        };

        let parent_id = state.definition_names[parent_name];
        let field_id = state.selection_map[&(parent_id, field.name.node.as_str())];
        let field_type_id = state.fields[field_id.0].field_type_id;

        let Some(subgraph_id) = state.fields[field_id.0].resolvable_in else {
            continue;
        };

        let provides = join_field_directive
            .node
            .get_argument("provides")
            .and_then(|arg| match &arg.node {
                async_graphql_value::ConstValue::String(s) => Some(s),
                _ => None,
            })
            .map(|provides| {
                parse_selection_set(provides)
                    .and_then(|provides| attach_selection(&provides, state.field_types[field_type_id.0].kind, state))
                    .map(|fields| vec![FieldProvides { subgraph_id, fields }])
            })
            .transpose()?
            .unwrap_or_default();

        let requires = join_field_directive
            .node
            .get_argument("requires")
            .and_then(|arg| match &arg.node {
                async_graphql_value::ConstValue::String(s) => Some(s),
                _ => None,
            })
            .map(|provides| {
                parse_selection_set(provides)
                    .and_then(|provides| attach_selection(&provides, parent_id, state))
                    .map(|fields| vec![FieldRequires { subgraph_id, fields }])
            })
            .transpose()?
            .unwrap_or_default();

        let field = &mut state.fields[field_id.0];
        field.provides = provides;
        field.requires = requires;
    }

    Ok(())
}

fn ingest_definitions<'a>(document: &'a ast::ServiceDocument, state: &mut State<'a>) -> Result<(), DomainError> {
    for definition in &document.definitions {
        match definition {
            ast::TypeSystemDefinition::Schema(_) | ast::TypeSystemDefinition::Directive(_) => (),
            ast::TypeSystemDefinition::Type(typedef) => {
                let type_name = typedef.node.name.node.as_str();
                let type_name_id = state.insert_string(type_name);
                let description = typedef
                    .node
                    .description
                    .as_ref()
                    .map(|description| state.insert_string(description.node.as_str()));
                let composed_directives = collect_composed_directives(&typedef.node.directives, state);

                match &typedef.node.kind {
                    ast::TypeKind::Scalar => {
                        let description = typedef
                            .node
                            .description
                            .as_ref()
                            .map(|description| state.insert_string(description.node.as_str()));

                        let scalar_id = ScalarId(state.scalars.push_return_idx(Scalar {
                            name: type_name_id,
                            composed_directives,
                            description,
                        }));
                        state.definition_names.insert(type_name, Definition::Scalar(scalar_id));
                    }
                    ast::TypeKind::Object(_) => {
                        let object_id = ObjectId(state.objects.push_return_idx(Object {
                            name: type_name_id,
                            implements_interfaces: Vec::new(),
                            resolvable_keys: Vec::new(),
                            composed_directives,
                            description,
                        }));

                        match type_name {
                            "Query" => state.query_type = Some(object_id),
                            "Mutation" => state.mutation_type = Some(object_id),
                            "Subscription" => state.subscription_type = Some(object_id),
                            _ => (),
                        }

                        state.definition_names.insert(type_name, Definition::Object(object_id));
                    }
                    ast::TypeKind::Interface(_) => {
                        let interface_id = InterfaceId(state.interfaces.push_return_idx(Interface {
                            name: type_name_id,
                            implements_interfaces: Vec::new(),
                            resolvable_keys: Vec::new(),
                            composed_directives,
                            description,
                        }));
                        state
                            .definition_names
                            .insert(type_name, Definition::Interface(interface_id));
                    }
                    ast::TypeKind::Union(_) => {
                        let union_id = UnionId(state.unions.push_return_idx(Union {
                            name: type_name_id,
                            members: Vec::new(),
                            composed_directives,
                            description,
                        }));
                        state.definition_names.insert(type_name, Definition::Union(union_id));
                    }
                    ast::TypeKind::Enum(enm) if type_name == JOIN_GRAPH_ENUM_NAME => {
                        ingest_join_graph_enum(enm, state)?;
                    }
                    ast::TypeKind::Enum(enm) => {
                        let enum_id = EnumId(state.enums.push_return_idx(Enum {
                            name: type_name_id,
                            values: Vec::new(),
                            composed_directives,
                            description,
                        }));
                        state.definition_names.insert(type_name, Definition::Enum(enum_id));

                        for value in &enm.values {
                            let composed_directives = collect_composed_directives(&value.node.directives, state);
                            let description = value
                                .node
                                .description
                                .as_ref()
                                .map(|description| state.insert_string(description.node.as_str()));
                            let value = state.insert_string(value.node.value.node.as_str());
                            state.enums[enum_id.0].values.push(EnumValue {
                                value,
                                composed_directives,
                                description,
                            });
                        }
                    }
                    ast::TypeKind::InputObject(_) => {
                        let input_object_id = InputObjectId(state.input_objects.push_return_idx(InputObject {
                            name: type_name_id,
                            fields: Vec::new(),
                            composed_directives,
                            description,
                        }));
                        state
                            .definition_names
                            .insert(type_name, Definition::InputObject(input_object_id));
                    }
                }
            }
        }
    }

    insert_builtin_scalars(state);

    Ok(())
}

fn insert_builtin_scalars(state: &mut State<'_>) {
    for name_str in ["String", "ID", "Float", "Boolean", "Int"] {
        let name = state.insert_string(name_str);
        let id = ScalarId(state.scalars.push_return_idx(Scalar {
            name,
            composed_directives: Vec::new(),
            description: None,
        }));
        state.definition_names.insert(name_str, Definition::Scalar(id));
    }
}

fn ingest_interface<'a>(interface_id: InterfaceId, iface: &'a ast::InterfaceType, state: &mut State<'a>) {
    for field in &iface.fields {
        let field_id = ingest_field(Definition::Interface(interface_id), &field.node, state);
        state.interface_fields.push(InterfaceField { interface_id, field_id });
    }
}

fn ingest_field<'a>(parent_id: Definition, ast_field: &'a ast::FieldDefinition, state: &mut State<'a>) -> FieldId {
    let field_name = ast_field.name.node.as_str();
    let field_type_id = state.insert_field_type(&ast_field.ty.node);
    let name = state.insert_string(field_name);
    let arguments = ast_field
        .arguments
        .iter()
        .map(|arg| {
            let description = arg
                .node
                .description
                .as_ref()
                .map(|description| state.insert_string(description.node.as_str()));
            let composed_directives = collect_composed_directives(&arg.node.directives, state);
            FieldArgument {
                name: state.insert_string(arg.node.name.node.as_str()),
                type_id: state.insert_field_type(&arg.node.ty.node),
                composed_directives,
                description,
            }
        })
        .collect();

    let resolvable_in = ast_field
        .directives
        .iter()
        .find(|dir| dir.node.name.node == JOIN_FIELD_DIRECTIVE_NAME)
        .filter(|dir| dir.node.get_argument("overrides").is_none())
        .and_then(|dir| dir.node.get_argument("graph"))
        .and_then(|arg| match &arg.node {
            async_graphql_value::ConstValue::Enum(s) => Some(state.graph_sdl_names[s.as_str()]),
            _ => None,
        });

    let overrides = ast_field
        .directives
        .iter()
        .filter(|dir| dir.node.name.node == JOIN_FIELD_DIRECTIVE_NAME)
        .filter_map(|dir| dir.node.get_argument("graph").zip(dir.node.get_argument("overrides")))
        .filter_map(|(graph, overrides)| match (&graph.node, &overrides.node) {
            (async_graphql_value::ConstValue::Enum(graph), async_graphql_value::ConstValue::String(overrides)) => {
                let subgraph_name = state.insert_string(graph.as_str());
                Some(Override {
                    graph: SubgraphId(
                        state
                            .subgraphs
                            .iter()
                            .position(|subgraph| subgraph.name == subgraph_name)?,
                    ),
                    from: state
                        .subgraphs
                        .iter()
                        .position(|subgraph| &state.strings[subgraph.name.0] == overrides)
                        .map(SubgraphId)
                        .map(OverrideSource::Subgraph)
                        .unwrap_or_else(|| OverrideSource::Missing(state.insert_string(overrides))),
                })
            }
            _ => None, // unreachable in valid schemas
        })
        .collect();

    let composed_directives = collect_composed_directives(&ast_field.directives, state);
    let description = ast_field
        .description
        .as_ref()
        .map(|description| state.insert_string(description.node.as_str()));

    let field_id = FieldId(state.fields.push_return_idx(Field {
        name,
        field_type_id,
        resolvable_in,
        provides: Vec::new(),
        requires: Vec::new(),
        arguments,
        composed_directives,
        overrides,
        description,
    }));

    state.selection_map.insert((parent_id, field_name), field_id);

    field_id
}

fn ingest_union_members<'a>(
    union_id: UnionId,
    union: &'a ast::UnionType,
    state: &mut State<'a>,
) -> Result<(), DomainError> {
    for member in &union.members {
        let Definition::Object(object_id) = state.definition_names[member.node.as_str()] else {
            return Err(DomainError("Non-object type in union members".to_owned()));
        };
        state.unions[union_id.0].members.push(object_id);
    }

    Ok(())
}

fn ingest_input_object<'a>(
    input_object_id: InputObjectId,
    input_object: &'a ast::InputObjectType,
    state: &mut State<'a>,
) {
    for field in &input_object.fields {
        let name = state.insert_string(field.node.name.node.as_str());
        let field_type_id = state.insert_field_type(&field.node.ty.node);
        let composed_directives = collect_composed_directives(&field.node.directives, state);
        let description = field
            .node
            .description
            .as_ref()
            .map(|description| state.insert_string(description.node.as_str()));
        state.input_objects[input_object_id.0].fields.push(InputObjectField {
            name,
            field_type_id,
            composed_directives,
            description,
        });
    }
}

fn ingest_object_fields<'a>(object_id: ObjectId, object: &'a ast::ObjectType, state: &mut State<'a>) {
    for field in &object.fields {
        let field_id = ingest_field(Definition::Object(object_id), &field.node, state);
        state.object_fields.push(ObjectField { object_id, field_id });
    }
}

fn parse_selection_set(fields: &str) -> Result<Vec<Positioned<ast::Selection>>, DomainError> {
    // Cheating for now, we should port the parser from engines instead.
    let fields = format!("{{ {fields} }}");
    let parsed = async_graphql_parser::parse_query(fields)
        .map_err(|err| err.to_string())
        .map_err(DomainError)?;

    let ast::ExecutableDocument {
        operations: ast::DocumentOperations::Single(operation),
        ..
    } = parsed
    else {
        return Err(DomainError(
            "The `fields` argument contents were not a valid selection set".to_owned(),
        ));
    };

    Ok(operation.node.selection_set.node.items)
}

/// Attach a selection set defined in strings to a FederatedGraph, transforming the strings into
/// field ids.
fn attach_selection(
    selection_set: &[Positioned<ast::Selection>],
    parent_id: Definition,
    state: &mut State<'_>,
) -> Result<FieldSet, DomainError> {
    selection_set
        .iter()
        .map(|selection| {
            let ast::Selection::Field(ast_field) = &selection.node else {
                return Err(DomainError("Unsupported fragment spread in selection set".to_owned()));
            };
            let field = state.selection_map[&(parent_id, ast_field.node.name.node.as_str())];
            let field_ty = state.field_types[state.fields[field.0].field_type_id.0].kind;
            let subselection = &ast_field.node.selection_set.node.items;
            Ok(FieldSetItem {
                field,
                subselection: attach_selection(subselection, field_ty, state)?,
            })
        })
        .collect()
}

fn ingest_join_graph_enum<'a>(enm: &'a ast::EnumType, state: &mut State<'a>) -> Result<(), DomainError> {
    for value in &enm.values {
        let sdl_name = value.node.value.node.as_str();
        let directive = value
            .node
            .directives
            .iter()
            .find(|directive| directive.node.name.node == JOIN_GRAPH_DIRECTIVE_NAME)
            .ok_or_else(|| DomainError("Missing @join__graph directive on join__Graph enum value.".to_owned()))?;
        let name = directive
            .node
            .get_argument("name")
            .ok_or_else(|| {
                DomainError(
                    "Missing `name` argument in `@join__graph` directive on `join__Graph` enum value.".to_owned(),
                )
            })
            .and_then(|arg| match &arg.node {
                async_graphql_value::ConstValue::String(s) => Ok(s),
                _ => Err(DomainError(
                    "Unexpected type for `name` argument in `@join__graph` directive on `join__Graph` enum value."
                        .to_owned(),
                )),
            })?;
        let url = directive
            .node
            .get_argument("url")
            .ok_or_else(|| {
                DomainError(
                    "Missing `url` argument in `@join__graph` directive on `join__Graph` enum value.".to_owned(),
                )
            })
            .and_then(|arg| match &arg.node {
                async_graphql_value::ConstValue::String(s) => Ok(s),
                _ => Err(DomainError(
                    "Unexpected type for `url` argument in `@join__graph` directive on `join__Graph` enum value."
                        .to_owned(),
                )),
            })?;

        let name = state.insert_string(name);
        let url = state.insert_string(url);
        let id = SubgraphId(state.subgraphs.push_return_idx(Subgraph { name, url }));
        state.graph_sdl_names.insert(sdl_name, id);
    }

    Ok(())
}

trait VecExt<T> {
    fn push_return_idx(&mut self, elem: T) -> usize;
}

impl<T> VecExt<T> for Vec<T> {
    fn push_return_idx(&mut self, elem: T) -> usize {
        let idx = self.len();
        self.push(elem);
        idx
    }
}

fn collect_composed_directives(
    directives: &[Positioned<ast::ConstDirective>],
    state: &mut State<'_>,
) -> Vec<Directive> {
    directives
        .iter()
        .filter(|dir| dir.node.name.node != JOIN_FIELD_DIRECTIVE_NAME)
        .filter(|dir| dir.node.name.node != JOIN_TYPE_DIRECTIVE_NAME)
        .map(|directive| Directive {
            name: state.insert_string(directive.node.name.node.as_str()),
            arguments: directive
                .node
                .arguments
                .iter()
                .map(|(name, value)| -> (StringId, Value) {
                    (state.insert_string(name.node.as_str()), state.insert_value(&value.node))
                })
                .collect(),
        })
        .collect()
}
