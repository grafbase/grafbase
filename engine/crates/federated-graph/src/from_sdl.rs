use crate::{federated_graph::*, FederatedGraph};
use async_graphql_parser::{
    types::{self as ast},
    Positioned,
};
use indexmap::IndexSet;
use std::{collections::HashMap, error::Error as StdError, fmt, ops::Range};

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
    interfaces: Vec<Interface>,
    fields: Vec<Field>,

    directives: Vec<Directive>,
    input_value_definitions: Vec<InputValueDefinition>,

    enums: Vec<Enum>,
    enum_values: Vec<EnumValue>,
    unions: Vec<Union>,
    scalars: Vec<Scalar>,
    input_objects: Vec<InputObject>,

    strings: IndexSet<String>,
    query_type_name: Option<String>,
    mutation_type_name: Option<String>,
    subscription_type_name: Option<String>,

    definition_names: HashMap<&'a str, Definition>,
    selection_map: HashMap<(Definition, &'a str), FieldId>,

    /// The key is the name of the graph in the join__Graph enum.
    graph_sdl_names: HashMap<&'a str, SubgraphId>,
}

impl<'a> State<'a> {
    fn field_type(&mut self, field_type: &'a ast::Type) -> Type {
        fn unfurl(state: &State<'_>, inner: &ast::Type) -> (wrapping::Wrapping, Definition) {
            match &inner.base {
                ast::BaseType::Named(name) => (
                    wrapping::Wrapping::new(!inner.nullable),
                    state.definition_names[name.as_str()],
                ),
                ast::BaseType::List(new_inner) => {
                    let (wrapping, definition) = unfurl(state, new_inner);
                    let wrapping = if inner.nullable {
                        wrapping.wrapped_by_nullable_list()
                    } else {
                        wrapping.wrapped_by_required_list()
                    };
                    (wrapping, definition)
                }
            }
        }

        let (wrapping, definition) = unfurl(self, field_type);

        Type { definition, wrapping }
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

    fn root_operation_types(&self) -> Result<RootOperationTypes, DomainError> {
        fn get_object_id(state: &State<'_>, name: &str) -> Option<ObjectId> {
            state
                .definition_names
                .get(name)
                .and_then(|definition| match definition {
                    Definition::Object(object_id) => Some(*object_id),
                    _ => None,
                })
        }
        let query_type_name = self.query_type_name.as_deref().unwrap_or("Query");
        let mutation_type_name = self.mutation_type_name.as_deref().unwrap_or("Mutation");
        let subscription_type_name = self.subscription_type_name.as_deref().unwrap_or("Subscription");
        Ok(RootOperationTypes {
            query: get_object_id(self, query_type_name)
                .ok_or_else(|| DomainError(format!("The `{query_type_name}` type is not defined")))?,
            mutation: get_object_id(self, mutation_type_name),
            subscription: get_object_id(self, subscription_type_name),
        })
    }
}

pub fn from_sdl(sdl: &str) -> Result<FederatedGraph, DomainError> {
    let mut state = State::default();
    let parsed = async_graphql_parser::parse_schema(sdl).map_err(|err| DomainError(err.to_string()))?;

    ingest_definitions(&parsed, &mut state)?;
    ingest_schema_definitions(&parsed, &mut state)?;

    // Ensure that the root query type is defined
    let query_type = state
        .definition_names
        .get(state.query_type_name.as_deref().unwrap_or("Query"));

    if query_type.is_none() {
        let query_type_name = "Query";
        state.query_type_name = Some(String::from(query_type_name));

        let object_id = ObjectId(state.objects.len());
        let query_string_id = state.insert_string(query_type_name);

        state
            .definition_names
            .insert(query_type_name, Definition::Object(object_id));

        state.objects.push(Object {
            name: query_string_id,
            implements_interfaces: vec![],
            keys: vec![],
            composed_directives: NO_DIRECTIVES,
            fields: NO_FIELDS,
            description: None,
        });

        ingest_object_fields(object_id, &[], &mut state);
    }

    ingest_fields(&parsed, &mut state)?;
    // This needs to happen after all fields have been ingested, in order to attach selection sets.
    ingest_selection_sets(&parsed, &mut state)?;

    Ok(FederatedGraph::V3(FederatedGraphV3 {
        root_operation_types: state.root_operation_types()?,
        subgraphs: state.subgraphs,
        objects: state.objects,
        interfaces: state.interfaces,
        fields: state.fields,
        enums: state.enums,
        enum_values: state.enum_values,
        unions: state.unions,
        scalars: state.scalars,
        input_objects: state.input_objects,
        strings: state.strings.into_iter().collect(),
        directives: state.directives,
        input_value_definitions: state.input_value_definitions,
    }))
}

fn ingest_schema_definitions<'a>(parsed: &'a ast::ServiceDocument, state: &mut State<'a>) -> Result<(), DomainError> {
    for definition in &parsed.definitions {
        if let ast::TypeSystemDefinition::Schema(Positioned { node: schema, .. }) = definition {
            ingest_schema_definition(schema, state)?;
        }
    }

    Ok(())
}

fn ingest_fields<'a>(parsed: &'a ast::ServiceDocument, state: &mut State<'a>) -> Result<(), DomainError> {
    for definition in &parsed.definitions {
        match definition {
            ast::TypeSystemDefinition::Schema(_) | ast::TypeSystemDefinition::Directive(_) => (),
            ast::TypeSystemDefinition::Type(typedef) => match &typedef.node.kind {
                ast::TypeKind::Scalar => (),
                ast::TypeKind::Object(object) => {
                    let Definition::Object(object_id) = state.definition_names[typedef.node.name.node.as_str()] else {
                        return Err(DomainError(
                            "Broken invariant: object id behind object name.".to_owned(),
                        ));
                    };
                    ingest_object_interfaces(object_id, object, state)?;
                    ingest_object_fields(object_id, &object.fields, state);
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

fn ingest_schema_definition(schema: &ast::SchemaDefinition, state: &mut State<'_>) -> Result<(), DomainError> {
    for Positioned { node: directive, .. } in &schema.directives {
        let name = directive.name.node.as_str();
        if name != "link" {
            return Err(DomainError(format!("Unsupported directive {name} on schema.")));
        }
    }

    if let Some(Positioned { node: name, .. }) = &schema.query {
        state.query_type_name = Some(name.to_string());
    }
    if let Some(Positioned { node: name, .. }) = &schema.mutation {
        state.mutation_type_name = Some(name.to_string());
    }
    if let Some(Positioned { node: name, .. }) = &schema.subscription {
        state.subscription_type_name = Some(name.to_string());
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
            let resolvable = join_type
                .node
                .get_argument("resolvable")
                .and_then(|arg| match &arg.node {
                    async_graphql_value::ConstValue::Boolean(b) => Some(*b),
                    _ => None,
                })
                .unwrap_or(true);

            let is_interface_object = join_type
                .node
                .get_argument("isInterfaceObject")
                .map(|arg| matches!(arg.node, async_graphql_value::ConstValue::Boolean(true)))
                .unwrap_or(false);

            match definition {
                Definition::Object(object_id) => {
                    state.objects[object_id.0].keys.push(Key {
                        subgraph_id,
                        fields,
                        is_interface_object,
                        resolvable,
                    });
                }
                Definition::Interface(interface_id) => {
                    state.interfaces[interface_id.0].keys.push(Key {
                        subgraph_id,
                        fields,
                        is_interface_object,
                        resolvable,
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
        let field_type = state.fields[field_id.0].r#type.clone();

        let Some(subgraph_id) = state.fields[field_id.0].resolvable_in.first().copied() else {
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
                    .and_then(|provides| attach_selection(&provides, field_type.definition, state))
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
                            keys: Vec::new(),
                            composed_directives,
                            description,
                            fields: NO_FIELDS,
                        }));

                        state.definition_names.insert(type_name, Definition::Object(object_id));
                    }
                    ast::TypeKind::Interface(_) => {
                        let interface_id = InterfaceId(state.interfaces.push_return_idx(Interface {
                            name: type_name_id,
                            implements_interfaces: Vec::new(),
                            keys: Vec::new(),
                            composed_directives,
                            description,
                            fields: NO_FIELDS,
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
                        let values = {
                            let start = state.enum_values.len();

                            for value in &enm.values {
                                let composed_directives = collect_composed_directives(&value.node.directives, state);
                                let description = value
                                    .node
                                    .description
                                    .as_ref()
                                    .map(|description| state.insert_string(description.node.as_str()));
                                let value = state.insert_string(value.node.value.node.as_str());
                                state.enum_values.push(EnumValue {
                                    value,
                                    composed_directives,
                                    description,
                                });
                            }

                            (EnumValueId(start), state.enum_values.len() - start)
                        };

                        let enum_id = EnumId(state.enums.push_return_idx(Enum {
                            name: type_name_id,
                            values,
                            composed_directives,
                            description,
                        }));
                        state.definition_names.insert(type_name, Definition::Enum(enum_id));
                    }
                    ast::TypeKind::InputObject(_) => {
                        let input_object_id = InputObjectId(state.input_objects.push_return_idx(InputObject {
                            name: type_name_id,
                            fields: NO_INPUT_VALUE_DEFINITION,
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
            composed_directives: (DirectiveId(0), 0),
            description: None,
        }));
        state.definition_names.insert(name_str, Definition::Scalar(id));
    }
}

fn ingest_interface<'a>(interface_id: InterfaceId, iface: &'a ast::InterfaceType, state: &mut State<'a>) {
    let [mut start, mut end] = [None; 2];

    for field in &iface.fields {
        let field_id = ingest_field(Definition::Interface(interface_id), &field.node, state);
        start = Some(start.unwrap_or(field_id));
        end = Some(field_id);
    }

    let [Some(start), Some(end)] = [start, end] else { return };
    state.interfaces[interface_id.0].fields = Range {
        start,
        end: FieldId(end.0 + 1),
    };
}

fn ingest_field<'a>(parent_id: Definition, ast_field: &'a ast::FieldDefinition, state: &mut State<'a>) -> FieldId {
    let field_name = ast_field.name.node.as_str();
    let r#type = state.field_type(&ast_field.ty.node);
    let name = state.insert_string(field_name);
    let args_start = state.input_value_definitions.len();

    for arg in &ast_field.arguments {
        let description = arg
            .node
            .description
            .as_ref()
            .map(|description| state.insert_string(description.node.as_str()));
        let composed_directives = collect_composed_directives(&arg.node.directives, state);
        let name = state.insert_string(arg.node.name.node.as_str());
        let r#type = state.field_type(&arg.node.ty.node);

        state.input_value_definitions.push(InputValueDefinition {
            name,
            r#type,
            directives: composed_directives,
            description,
        });
    }

    let args_end = state.input_value_definitions.len();

    let resolvable_in = ast_field
        .directives
        .iter()
        .filter(|dir| dir.node.name.node == JOIN_FIELD_DIRECTIVE_NAME)
        .filter(|dir| dir.node.get_argument("overrides").is_none())
        .filter(|dir| {
            !dir.node
                .get_argument("external")
                .map(|arg| match &arg.node {
                    async_graphql_value::ConstValue::Boolean(b) => *b,
                    _ => false,
                })
                .unwrap_or_default()
        })
        .filter_map(|dir| dir.node.get_argument("graph"))
        .filter_map(|arg| match &arg.node {
            async_graphql_value::ConstValue::Enum(s) => Some(state.graph_sdl_names[s.as_str()]),
            _ => None,
        })
        .collect();

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
        r#type,
        resolvable_in,
        provides: Vec::new(),
        requires: Vec::new(),
        arguments: (InputValueDefinitionId(args_start), args_end - args_start),
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
    let start = state.input_value_definitions.len();
    for field in &input_object.fields {
        let name = state.insert_string(field.node.name.node.as_str());
        let r#type = state.field_type(&field.node.ty.node);
        let composed_directives = collect_composed_directives(&field.node.directives, state);
        let description = field
            .node
            .description
            .as_ref()
            .map(|description| state.insert_string(description.node.as_str()));
        state.input_value_definitions.push(InputValueDefinition {
            name,
            r#type,
            directives: composed_directives,
            description,
        });
    }
    let end = state.input_value_definitions.len();

    state.input_objects[input_object_id.0].fields = (InputValueDefinitionId(start), end - start);
}

fn ingest_object_fields<'a>(
    object_id: ObjectId,
    fields: &'a [Positioned<ast::FieldDefinition>],
    state: &mut State<'a>,
) {
    let [mut start, mut end] = [None; 2];

    for field in fields {
        let field_id = ingest_field(Definition::Object(object_id), &field.node, state);
        start = Some(start.unwrap_or(field_id));
        end = Some(FieldId(field_id.0 + 1));
    }

    // When we encounter the root query type, we need to make space at the end of the fields for __type and __schema.
    if object_id
        == state
            .root_operation_types()
            .expect("root operation types to be defined at this point")
            .query
    {
        let new_start = state.fields.len();

        for name in ["__schema", "__type"].map(|name| state.insert_string(name)) {
            state.fields.push(Field {
                name,
                r#type: Type {
                    wrapping: Wrapping::new(false),
                    definition: Definition::Object(object_id),
                },
                arguments: NO_INPUT_VALUE_DEFINITION,
                resolvable_in: Vec::new(),
                provides: Vec::new(),
                requires: Vec::new(),
                overrides: Vec::new(),
                composed_directives: NO_DIRECTIVES,
                description: None,
            });
        }

        start = start.or(Some(FieldId(new_start)));
        end = end.map(|end| FieldId(end.0 + 2)).or(Some(FieldId(new_start + 2)));
    }

    let [Some(start), Some(end)] = [start, end] else {
        return;
    };

    state.objects[object_id.0].fields = Range { start, end };
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
            let field: FieldId = state.selection_map[&(parent_id, ast_field.node.name.node.as_str())];
            let field_ty = state.fields[field.0].r#type.definition;
            let subselection = &ast_field.node.selection_set.node.items;
            let arguments = ast_field
                .node
                .arguments
                .iter()
                .map(|(name, value)| {
                    let name = state.insert_string(&name.node);
                    let (start, len) = state.fields[field.0].arguments;
                    let arguments = &state.input_value_definitions[start.0..start.0 + len];
                    let argument = arguments
                        .iter()
                        .position(|arg| arg.name == name)
                        .map(|idx| InputValueDefinitionId(start.0 + idx))
                        .expect("unknown argument");
                    let value = state.insert_value(&value.node.clone().into_const().expect("Value -> ConstValue"));
                    (argument, value)
                })
                .collect();
            Ok(FieldSetItem {
                field,
                arguments,
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

fn collect_composed_directives(directives: &[Positioned<ast::ConstDirective>], state: &mut State<'_>) -> Directives {
    let start = state.directives.len();

    for directive in directives
        .iter()
        .filter(|dir| dir.node.name.node != JOIN_FIELD_DIRECTIVE_NAME)
        .filter(|dir| dir.node.name.node != JOIN_TYPE_DIRECTIVE_NAME)
    {
        match directive.node.name.node.as_str() {
            "inaccessible" => state.directives.push(Directive::Inaccessible),
            "deprecated" => {
                let directive = Directive::Deprecated {
                    reason: directive
                        .node
                        .get_argument("reason")
                        .and_then(|value| match &value.node {
                            async_graphql_value::ConstValue::String(s) => Some(state.insert_string(s.as_str())),
                            _ => None,
                        }),
                };

                state.directives.push(directive)
            }
            "requiresScopes" => {
                let scopes: Option<Vec<Vec<String>>> = directive
                    .node
                    .get_argument("scopes")
                    .and_then(|scopes| scopes.node.clone().into_json().ok())
                    .and_then(|scopes| serde_json::from_value(scopes).ok());

                if let Some(scopes) = scopes {
                    let transformed = scopes
                        .into_iter()
                        .map(|scopes| scopes.into_iter().map(|scope| state.insert_string(&scope)).collect())
                        .collect();
                    state.directives.push(Directive::RequiresScopes(transformed));
                }
            }
            "policy" => {
                let policies: Option<Vec<Vec<String>>> = directive
                    .node
                    .get_argument("policies")
                    .and_then(|policies| policies.node.clone().into_json().ok())
                    .and_then(|policies| serde_json::from_value(policies).ok());

                if let Some(policies) = policies {
                    let transformed = policies
                        .into_iter()
                        .map(|policies| {
                            policies
                                .into_iter()
                                .map(|policy| state.insert_string(&policy))
                                .collect()
                        })
                        .collect();
                    state.directives.push(Directive::Policy(transformed));
                }
            }
            other => {
                let name = state.insert_string(other);
                let arguments = directive
                    .node
                    .arguments
                    .iter()
                    .map(|(name, value)| -> (StringId, Value) {
                        (state.insert_string(name.node.as_str()), state.insert_value(&value.node))
                    })
                    .collect();

                state.directives.push(Directive::Other { name, arguments })
            }
        }
    }

    (DirectiveId(start), state.directives.len() - start)
}

#[cfg(test)]
#[test]
fn test_from_sdl() {
    // https://github.com/the-guild-org/gateways-benchmark/blob/main/federation-v1/gateways/apollo-router/supergraph.graphql
    let schema = super::from_sdl(r#"
        schema
          @link(url: "https://specs.apollo.dev/link/v1.0")
          @link(url: "https://specs.apollo.dev/join/v0.3", for: EXECUTION)
        {
          query: Query
        }

        directive @join__enumValue(graph: join__Graph!) repeatable on ENUM_VALUE

        directive @join__field(graph: join__Graph, requires: join__FieldSet, provides: join__FieldSet, type: String, external: Boolean, override: String, usedOverridden: Boolean) repeatable on FIELD_DEFINITION | INPUT_FIELD_DEFINITION

        directive @join__graph(name: String!, url: String!) on ENUM_VALUE

        directive @join__implements(graph: join__Graph!, interface: String!) repeatable on OBJECT | INTERFACE

        directive @join__type(graph: join__Graph!, key: join__FieldSet, extension: Boolean! = false, resolvable: Boolean! = true, isInterfaceObject: Boolean! = false) repeatable on OBJECT | INTERFACE | UNION | ENUM | INPUT_OBJECT | SCALAR

        directive @join__unionMember(graph: join__Graph!, member: String!) repeatable on UNION

        directive @link(url: String, as: String, for: link__Purpose, import: [link__Import]) repeatable on SCHEMA

        scalar join__FieldSet

        enum join__Graph {
          ACCOUNTS @join__graph(name: "accounts", url: "http://accounts:4001/graphql")
          INVENTORY @join__graph(name: "inventory", url: "http://inventory:4002/graphql")
          PRODUCTS @join__graph(name: "products", url: "http://products:4003/graphql")
          REVIEWS @join__graph(name: "reviews", url: "http://reviews:4004/graphql")
        }

        scalar link__Import

        enum link__Purpose {
          """
          `SECURITY` features provide metadata necessary to securely resolve fields.
          """
          SECURITY

          """
          `EXECUTION` features provide metadata necessary for operation execution.
          """
          EXECUTION
        }

        type Product
          @join__type(graph: INVENTORY, key: "upc")
          @join__type(graph: PRODUCTS, key: "upc")
          @join__type(graph: REVIEWS, key: "upc")
        {
          upc: String!
          weight: Int @join__field(graph: INVENTORY, external: true) @join__field(graph: PRODUCTS)
          price: Int @join__field(graph: INVENTORY, external: true) @join__field(graph: PRODUCTS)
          inStock: Boolean @join__field(graph: INVENTORY)
          shippingEstimate: Int @join__field(graph: INVENTORY, requires: "price weight")
          name: String @join__field(graph: PRODUCTS)
          reviews: [Review] @join__field(graph: REVIEWS)
        }

        type Query
          @join__type(graph: ACCOUNTS)
          @join__type(graph: INVENTORY)
          @join__type(graph: PRODUCTS)
          @join__type(graph: REVIEWS)
        {
          me: User @join__field(graph: ACCOUNTS)
          user(id: ID!): User @join__field(graph: ACCOUNTS)
          users: [User] @join__field(graph: ACCOUNTS)
          topProducts(first: Int = 5): [Product] @join__field(graph: PRODUCTS)
        }

        type Review
          @join__type(graph: REVIEWS, key: "id")
        {
          id: ID!
          body: String
          product: Product
          author: User @join__field(graph: REVIEWS, provides: "username")
        }

        type User
          @join__type(graph: ACCOUNTS, key: "id")
          @join__type(graph: REVIEWS, key: "id")
        {
          id: ID!
          name: String @join__field(graph: ACCOUNTS)
          username: String @join__field(graph: ACCOUNTS) @join__field(graph: REVIEWS, external: true)
          birthday: Int @join__field(graph: ACCOUNTS)
          reviews: [Review] @join__field(graph: REVIEWS)
        }
    "#).unwrap();

    let schema = schema.into_latest();
    let query_object = &schema[schema.root_operation_types.query];

    for field_name in ["__type", "__schema"] {
        let field_name = schema.strings.iter().position(|s| s == field_name).unwrap();
        assert!(schema[query_object.fields.clone()]
            .iter()
            .any(|f| f.name.0 == field_name));
    }
}

#[cfg(test)]
#[test]
fn test_from_sdl_with_empty_query_root() {
    // https://github.com/the-guild-org/gateways-benchmark/blob/main/federation-v1/gateways/apollo-router/supergraph.graphql
    let schema = super::from_sdl(
        r#"
        schema
          @link(url: "https://specs.apollo.dev/link/v1.0")
          @link(url: "https://specs.apollo.dev/join/v0.3", for: EXECUTION)
        {
          query: Query
        }

        directive @join__enumValue(graph: join__Graph!) repeatable on ENUM_VALUE

        directive @join__field(graph: join__Graph, requires: join__FieldSet, provides: join__FieldSet, type: String, external: Boolean, override: String, usedOverridden: Boolean) repeatable on FIELD_DEFINITION | INPUT_FIELD_DEFINITION

        directive @join__graph(name: String!, url: String!) on ENUM_VALUE

        directive @join__implements(graph: join__Graph!, interface: String!) repeatable on OBJECT | INTERFACE

        directive @join__type(graph: join__Graph!, key: join__FieldSet, extension: Boolean! = false, resolvable: Boolean! = true, isInterfaceObject: Boolean! = false) repeatable on OBJECT | INTERFACE | UNION | ENUM | INPUT_OBJECT | SCALAR

        directive @join__unionMember(graph: join__Graph!, member: String!) repeatable on UNION

        directive @link(url: String, as: String, for: link__Purpose, import: [link__Import]) repeatable on SCHEMA

        scalar join__FieldSet

        enum join__Graph {
          ACCOUNTS @join__graph(name: "accounts", url: "http://accounts:4001/graphql")
          INVENTORY @join__graph(name: "inventory", url: "http://inventory:4002/graphql")
          PRODUCTS @join__graph(name: "products", url: "http://products:4003/graphql")
          REVIEWS @join__graph(name: "reviews", url: "http://reviews:4004/graphql")
        }

        scalar link__Import

        enum link__Purpose {
          """
          `SECURITY` features provide metadata necessary to securely resolve fields.
          """
          SECURITY

          """
          `EXECUTION` features provide metadata necessary for operation execution.
          """
          EXECUTION
        }

        type Query

        type User
          @join__type(graph: ACCOUNTS, key: "id")
          @join__type(graph: REVIEWS, key: "id")
        {
          id: ID!
          name: String @join__field(graph: ACCOUNTS)
          username: String @join__field(graph: ACCOUNTS) @join__field(graph: REVIEWS, external: true)
          birthday: Int @join__field(graph: ACCOUNTS)
          reviews: [Review] @join__field(graph: REVIEWS)
        }

        type Review
          @join__type(graph: REVIEWS, key: "id")
        {
          id: ID!
          body: String
          author: User @join__field(graph: REVIEWS, provides: "username")
        }
    "#,
    ).unwrap();

    let schema = schema.into_latest();
    let query_object = &schema[schema.root_operation_types.query];

    for field_name in ["__type", "__schema"] {
        let field_name = schema.strings.iter().position(|s| s == field_name).unwrap();
        assert!(schema[query_object.fields.clone()]
            .iter()
            .any(|f| f.name.0 == field_name));
    }
}

#[cfg(test)]
#[test]
fn test_from_sdl_with_missing_query_root() {
    // https://github.com/the-guild-org/gateways-benchmark/blob/main/federation-v1/gateways/apollo-router/supergraph.graphql
    let schema = super::from_sdl(
        r#"
        schema
          @link(url: "https://specs.apollo.dev/link/v1.0")
          @link(url: "https://specs.apollo.dev/join/v0.3", for: EXECUTION)
        {
          query: Query
        }

        directive @join__enumValue(graph: join__Graph!) repeatable on ENUM_VALUE

        directive @join__field(graph: join__Graph, requires: join__FieldSet, provides: join__FieldSet, type: String, external: Boolean, override: String, usedOverridden: Boolean) repeatable on FIELD_DEFINITION | INPUT_FIELD_DEFINITION

        directive @join__graph(name: String!, url: String!) on ENUM_VALUE

        directive @join__implements(graph: join__Graph!, interface: String!) repeatable on OBJECT | INTERFACE

        directive @join__type(graph: join__Graph!, key: join__FieldSet, extension: Boolean! = false, resolvable: Boolean! = true, isInterfaceObject: Boolean! = false) repeatable on OBJECT | INTERFACE | UNION | ENUM | INPUT_OBJECT | SCALAR

        directive @join__unionMember(graph: join__Graph!, member: String!) repeatable on UNION

        directive @link(url: String, as: String, for: link__Purpose, import: [link__Import]) repeatable on SCHEMA

        scalar join__FieldSet

        enum join__Graph {
          ACCOUNTS @join__graph(name: "accounts", url: "http://accounts:4001/graphql")
          INVENTORY @join__graph(name: "inventory", url: "http://inventory:4002/graphql")
          PRODUCTS @join__graph(name: "products", url: "http://products:4003/graphql")
          REVIEWS @join__graph(name: "reviews", url: "http://reviews:4004/graphql")
        }

        scalar link__Import

        enum link__Purpose {
          """
          `SECURITY` features provide metadata necessary to securely resolve fields.
          """
          SECURITY

          """
          `EXECUTION` features provide metadata necessary for operation execution.
          """
          EXECUTION
        }

        type Review
          @join__type(graph: REVIEWS, key: "id")
        {
          id: ID!
          body: String
          author: User @join__field(graph: REVIEWS, provides: "username")
        }

        type User
          @join__type(graph: ACCOUNTS, key: "id")
          @join__type(graph: REVIEWS, key: "id")
        {
          id: ID!
          name: String @join__field(graph: ACCOUNTS)
          username: String @join__field(graph: ACCOUNTS) @join__field(graph: REVIEWS, external: true)
          birthday: Int @join__field(graph: ACCOUNTS)
          reviews: [Review] @join__field(graph: REVIEWS)
        }
    "#,
    ).unwrap();

    let schema = schema.into_latest();
    let query_object = &schema[schema.root_operation_types.query];

    for field_name in ["__type", "__schema"] {
        let field_name = schema.strings.iter().position(|s| s == field_name).unwrap();
        assert!(schema[query_object.fields.clone()]
            .iter()
            .any(|f| f.name.0 == field_name));
    }
}
