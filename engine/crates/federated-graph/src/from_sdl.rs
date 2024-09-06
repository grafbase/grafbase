mod arguments;
mod value;

use self::{arguments::*, value::*};
use crate::federated_graph::*;
use cynic_parser::{common::WrappingType, executable as executable_ast, type_system as ast};
use indexmap::IndexSet;
use std::{collections::HashMap, error::Error as StdError, fmt, ops::Range};

const JOIN_FIELD_DIRECTIVE_NAME: &str = "join__field";
const JOIN_FIELD_DIRECTIVE_OVERRIDE_ARGUMENT: &str = "override";
const JOIN_FIELD_DIRECTIVE_OVERRIDE_LABEL_ARGUMENT: &str = "overrideLabel";
const JOIN_GRAPH_DIRECTIVE_NAME: &str = "join__graph";
const JOIN_GRAPH_ENUM_NAME: &str = "join__Graph";
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
    input_values_map: HashMap<(InputObjectId, &'a str), InputValueDefinitionId>,
    enum_values_map: HashMap<(EnumId, &'a str), EnumValueId>,

    /// The key is the name of the graph in the join__Graph enum.
    graph_sdl_names: HashMap<&'a str, SubgraphId>,

    authorized_directives: Vec<AuthorizedDirective>,
    field_authorized_directives: Vec<(FieldId, AuthorizedDirectiveId)>,
    object_authorized_directives: Vec<(ObjectId, AuthorizedDirectiveId)>,
    interface_authorized_directives: Vec<(InterfaceId, AuthorizedDirectiveId)>,

    type_wrappers: Vec<WrappingType>,
}

impl<'a> State<'a> {
    fn field_type(&mut self, field_type: ast::Type<'a>) -> Result<Type, DomainError> {
        use cynic_parser::common::WrappingType;

        self.type_wrappers.clear();
        self.type_wrappers.extend(field_type.wrappers());
        self.type_wrappers.reverse();

        let mut wrappers = self.type_wrappers.iter().peekable();

        let mut wrapping = match wrappers.next() {
            Some(WrappingType::List) => wrapping::Wrapping::new(false).wrapped_by_nullable_list(),
            Some(WrappingType::NonNull) => wrapping::Wrapping::new(true),
            None => wrapping::Wrapping::new(false),
        };

        while let Some(next) = wrappers.next() {
            debug_assert_eq!(*next, WrappingType::List, "double non-null wrapping type not possible");

            wrapping = match wrappers.peek() {
                Some(WrappingType::NonNull) => {
                    wrappers.next();
                    wrapping.wrapped_by_required_list()
                }
                None | Some(WrappingType::List) => wrapping.wrapped_by_nullable_list(),
            }
        }

        let definition = *self
            .definition_names
            .get(field_type.name())
            .ok_or_else(|| DomainError(format!("Unknown type '{}'", field_type.name())))?;

        Ok(Type { definition, wrapping })
    }

    fn insert_string(&mut self, s: &str) -> StringId {
        if let Some(idx) = self.strings.get_index_of(s) {
            return StringId(idx);
        }

        StringId(self.strings.insert_full(s.to_owned()).0)
    }

    fn insert_value(&mut self, node: ast::Value<'_>, expected_enum_type: Option<EnumId>) -> Value {
        match node {
            ast::Value::Null => Value::String(self.insert_string("null")),
            ast::Value::Int(n) => Value::Int(i64::from(n)),
            ast::Value::Float(n) => Value::Float(f64::from(n)),
            ast::Value::String(s) | ast::Value::BlockString(s) => Value::String(self.insert_string(s)),
            ast::Value::Boolean(b) => Value::Boolean(b),
            ast::Value::Enum(enm) => expected_enum_type
                .and_then(|enum_id| {
                    let enum_value_id = self.enum_values_map.get(&(enum_id, enm))?;
                    Some(Value::EnumValue(*enum_value_id))
                })
                .unwrap_or(Value::UnboundEnumValue(self.insert_string(enm))),
            ast::Value::List(list) => Value::List(
                list.into_iter()
                    .map(|value| self.insert_value(value, expected_enum_type))
                    .collect(),
            ),
            ast::Value::Object(obj) => Value::Object(
                obj.into_iter()
                    .map(|(k, v)| (self.insert_string(k), self.insert_value(v, expected_enum_type)))
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            ),
            ast::Value::Variable(_) => unreachable!("variable in type system document"), // not possible in type system documents
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

    fn get_definition_name(&self, definition: Definition) -> &str {
        let name = match definition {
            Definition::Object(object_id) => self.objects[object_id.0].name,
            Definition::Interface(interface_id) => self.interfaces[interface_id.0].name,
            Definition::Scalar(scalar_id) => self.scalars[scalar_id.0].name,
            Definition::Enum(enum_id) => self.enums[enum_id.0].name,
            Definition::Union(union_id) => self.unions[union_id.0].name,
            Definition::InputObject(input_object_id) => self.input_objects[input_object_id.0].name,
        };
        &self.strings[name.0]
    }
}

pub fn from_sdl(sdl: &str) -> Result<FederatedGraph, DomainError> {
    let mut state = State::default();
    let parsed = cynic_parser::parse_type_system_document(sdl).map_err(|err| DomainError(err.to_string()))?;

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

        ingest_object_fields(object_id, std::iter::empty(), &mut state)?;
    }

    ingest_fields(&parsed, &mut state)?;
    // This needs to happen after all fields have been ingested, in order to attach selection sets.
    ingest_selection_sets(&parsed, &mut state)?;

    Ok(FederatedGraph {
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
        authorized_directives: state.authorized_directives,
        field_authorized_directives: state.field_authorized_directives,
        object_authorized_directives: state.object_authorized_directives,
        interface_authorized_directives: state.interface_authorized_directives,
    })
}

fn ingest_schema_definitions<'a>(
    parsed: &'a ast::TypeSystemDocument,
    state: &mut State<'a>,
) -> Result<(), DomainError> {
    for definition in parsed.definitions() {
        if let ast::Definition::Schema(schema) = definition {
            ingest_schema_definition(schema, state)?;
        }
    }

    Ok(())
}

fn ingest_fields<'a>(parsed: &'a ast::TypeSystemDocument, state: &mut State<'a>) -> Result<(), DomainError> {
    for definition in parsed.definitions() {
        match definition {
            ast::Definition::Schema(_) | ast::Definition::SchemaExtension(_) | ast::Definition::Directive(_) => (),
            ast::Definition::Type(typedef) | ast::Definition::TypeExtension(typedef) => match &typedef {
                ast::TypeDefinition::Scalar(_) => (),
                ast::TypeDefinition::Object(object) => {
                    let Definition::Object(object_id) = state.definition_names[typedef.name()] else {
                        return Err(DomainError(
                            "Broken invariant: object id behind object name.".to_owned(),
                        ));
                    };
                    ingest_object_interfaces(object_id, object, state)?;
                    ingest_object_fields(object_id, object.fields(), state)?;
                }
                ast::TypeDefinition::Interface(iface) => {
                    let Definition::Interface(interface_id) = state.definition_names[typedef.name()] else {
                        return Err(DomainError(
                            "Broken invariant: interface id behind interface name.".to_owned(),
                        ));
                    };
                    ingest_interface_interfaces(interface_id, iface, state)?;
                    ingest_interface_fields(interface_id, iface.fields(), state)?;
                }
                ast::TypeDefinition::Union(union) => {
                    let Definition::Union(union_id) = state.definition_names[typedef.name()] else {
                        return Err(DomainError("Broken invariant: UnionId behind union name.".to_owned()));
                    };
                    ingest_union_members(union_id, union, state)?;
                }
                ast::TypeDefinition::Enum(_) => {}
                ast::TypeDefinition::InputObject(input_object) => {
                    let Definition::InputObject(input_object_id) = state.definition_names[typedef.name()] else {
                        return Err(DomainError(
                            "Broken invariant: InputObjectId behind input object name.".to_owned(),
                        ));
                    };
                    ingest_input_object(input_object_id, input_object, state)?;
                }
            },
        }
    }

    Ok(())
}

fn ingest_schema_definition(schema: ast::SchemaDefinition<'_>, state: &mut State<'_>) -> Result<(), DomainError> {
    for directive in schema.directives() {
        let name = directive.name();
        if name != "link" {
            return Err(DomainError(format!("Unsupported directive {name} on schema.")));
        }
    }

    if let Some(query) = schema.query_type() {
        state.query_type_name = Some(query.named_type().to_owned());
    }

    if let Some(mutation) = schema.mutation_type() {
        state.mutation_type_name = Some(mutation.named_type().to_owned());
    }

    if let Some(subscription) = schema.subscription_type() {
        state.subscription_type_name = Some(subscription.named_type().to_owned());
    }

    Ok(())
}

fn ingest_interface_interfaces(
    interface_id: InterfaceId,
    interface: &ast::InterfaceDefinition<'_>,
    state: &mut State<'_>,
) -> Result<(), DomainError> {
    state.interfaces[interface_id.0].implements_interfaces = interface
        .implements_interfaces()
        .map(|name| match state.definition_names[name] {
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
    object: &ast::ObjectDefinition<'_>,
    state: &mut State<'_>,
) -> Result<(), DomainError> {
    state.objects[object_id.0].implements_interfaces = object
        .implements_interfaces()
        .map(|name| match state.definition_names[name] {
            Definition::Interface(interface_id) => Ok(interface_id),
            _ => Err(DomainError(
                "Broken invariant: object implements non-interface type".to_owned(),
            )),
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(())
}

fn ingest_selection_sets<'a>(parsed: &'a ast::TypeSystemDocument, state: &mut State<'a>) -> Result<(), DomainError> {
    ingest_field_directives_after_graph(parsed, state)?;
    ingest_authorized_directives(parsed, state)?;
    ingest_entity_keys(parsed, state)
}

fn ingest_authorized_directives(parsed: &ast::TypeSystemDocument, state: &mut State<'_>) -> Result<(), DomainError> {
    for typedef in parsed.definitions().filter_map(|def| match def {
        ast::Definition::Type(ty) => Some(ty),
        _ => None,
    }) {
        let Some(authorized) = typedef.directives().find(|directive| directive.name() == "authorized") else {
            continue;
        };

        let Some(definition) = state.definition_names.get(typedef.name()).copied() else {
            continue;
        };

        let fields = authorized
            .get_argument("fields")
            .and_then(|arg| match arg.value() {
                ast::Value::String(s) => Some(s),
                _ => None,
            })
            .map(|fields| parse_selection_set(fields).and_then(|doc| attach_field_set(&doc, definition, state)))
            .transpose()?;

        let metadata = authorized
            .get_argument("metadata")
            .map(|metadata| state.insert_value(metadata.value(), None));

        let idx = state.authorized_directives.push_return_idx(AuthorizedDirective {
            fields,
            node: None,
            arguments: None,
            metadata,
        });

        match definition {
            Definition::Object(object_id) => {
                state
                    .object_authorized_directives
                    .push((object_id, AuthorizedDirectiveId(idx)));
            }
            Definition::Interface(interface_id) => {
                state
                    .interface_authorized_directives
                    .push((interface_id, AuthorizedDirectiveId(idx)));
            }
            _ => (),
        }
    }

    Ok(())
}

fn ingest_entity_keys(parsed: &ast::TypeSystemDocument, state: &mut State<'_>) -> Result<(), DomainError> {
    for typedef in parsed.definitions().filter_map(|def| match def {
        ast::Definition::Type(ty) => Some(ty),
        _ => None,
    }) {
        let Some(definition) = state.definition_names.get(typedef.name()).copied() else {
            continue;
        };
        for join_type in typedef
            .directives()
            .filter(|dir| dir.name() == JOIN_TYPE_DIRECTIVE_NAME)
        {
            let subgraph_id = join_type
                .get_argument("graph")
                .and_then(|arg| match arg.value() {
                    ast::Value::Enum(s) => Some(state.graph_sdl_names[s]),
                    _ => None,
                })
                .expect("Missing graph argument in @join__type");
            let fields = join_type
                .get_argument("key")
                .and_then(|arg| match arg.value() {
                    ast::Value::String(s) => Some(s),
                    _ => None,
                })
                .map(|fields| parse_selection_set(fields).and_then(|doc| attach_field_set(&doc, definition, state)))
                .transpose()?
                .unwrap_or_default();
            let resolvable = join_type
                .get_argument("resolvable")
                .and_then(|arg| match arg.value() {
                    ast::Value::Boolean(b) => Some(b),
                    _ => None,
                })
                .unwrap_or(true);

            let is_interface_object = join_type
                .get_argument("isInterfaceObject")
                .map(|arg| matches!(arg.value(), ast::Value::Boolean(true)))
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

fn ingest_field_directives_after_graph<'a>(
    parsed: &'a ast::TypeSystemDocument,
    state: &mut State<'a>,
) -> Result<(), DomainError> {
    for definition in parsed.definitions() {
        let ast::Definition::Type(typedef) = definition else {
            continue;
        };
        let parent_id = state.definition_names[typedef.name()];

        let fields = match typedef {
            ast::TypeDefinition::Object(object) => object.fields(),
            ast::TypeDefinition::Interface(iface) => iface.fields(),
            _ => continue,
        };

        ingest_join_field_directive(parent_id, || typedef.directives(), fields, state)?;
        ingest_authorized_directive(parent_id, fields, state)?;
    }

    Ok(())
}

fn ingest_join_field_directive<'a, I>(
    parent_id: Definition,
    parent_directives: impl Fn() -> I,
    fields: impl Iterator<Item = ast::FieldDefinition<'a>>,
    state: &mut State<'_>,
) -> Result<(), DomainError>
where
    I: Iterator<Item = ast::Directive<'a>>,
{
    let is_federated_entity = parent_directives()
        .filter(|dir| dir.name() == JOIN_TYPE_DIRECTIVE_NAME)
        .any(|dir| dir.get_argument("key").is_some());

    let type_subgraph_ids = if !is_federated_entity {
        parent_directives()
            .filter(|dir| dir.name() == JOIN_TYPE_DIRECTIVE_NAME)
            .filter_map(|dir| {
                dir.get_argument("graph").and_then(|arg| match &arg.value() {
                    ast::Value::Enum(s) => Some(state.graph_sdl_names[s]),
                    _ => None,
                })
            })
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    for field in fields {
        let field_id = state.selection_map[&(parent_id, field.name())];
        let field_type = state.fields[field_id.0].r#type.clone();

        let mut resolvable_in = Vec::new();
        let mut requires = Vec::new();
        let mut provides = Vec::new();

        for directive in field.directives().filter(|dir| dir.name() == JOIN_FIELD_DIRECTIVE_NAME) {
            let is_external = directive
                .get_argument("external")
                .map(|arg| match arg.value() {
                    ast::Value::Boolean(b) => b,
                    _ => false,
                })
                .unwrap_or_default();

            if is_external {
                continue;
            }

            let Some(subgraph_id) = directive.get_argument("graph").and_then(|arg| match arg.value() {
                ast::Value::Enum(s) => state.graph_sdl_names.get(s).copied(),
                _ => None,
            }) else {
                continue;
            };

            // Overrides are handled in a completely different way
            if directive
                .get_argument(JOIN_FIELD_DIRECTIVE_OVERRIDE_ARGUMENT)
                .is_none() &&
                // We implemented "overrides" by mistake, so we allow it for backwards compatibility..
                directive.get_argument("overrides").is_none()
            {
                resolvable_in.push(subgraph_id);
            }

            if let Some(field_provides) = directive
                .get_argument("provides")
                .and_then(|arg| match arg.value() {
                    ast::Value::String(s) => Some(s),
                    _ => None,
                })
                .map(|provides| {
                    parse_selection_set(provides)
                        .and_then(|doc| attach_field_set(&doc, field_type.definition, state))
                        .map(|fields| FieldProvides { subgraph_id, fields })
                })
                .transpose()?
            {
                provides.push(field_provides)
            }

            if let Some(field_requires) = directive
                .get_argument("requires")
                .and_then(|arg| match arg.value() {
                    ast::Value::String(s) => Some(s),
                    _ => None,
                })
                .map(|requires| {
                    parse_selection_set(requires)
                        .and_then(|doc| attach_field_set(&doc, parent_id, state))
                        .map(|fields| FieldRequires { subgraph_id, fields })
                })
                .transpose()?
            {
                requires.push(field_requires);
            }
        }

        if resolvable_in.is_empty() {
            resolvable_in = type_subgraph_ids.clone();
        }

        let field = &mut state.fields[field_id.0];
        field.provides = provides;
        field.requires = requires;
        field.resolvable_in = resolvable_in;
    }

    Ok(())
}

fn ingest_authorized_directive<'a>(
    parent_id: Definition,
    fields: impl Iterator<Item = ast::FieldDefinition<'a>>,
    state: &mut State<'a>,
) -> Result<(), DomainError> {
    for field in fields {
        let field_id = state.selection_map[&(parent_id, field.name())];
        let field_type = state.fields[field_id.0].r#type.clone();

        for directive in field.directives() {
            if "authorized" != directive.name() {
                continue;
            }
            let authorized_directive = AuthorizedDirective {
                arguments: directive
                    .get_argument("arguments")
                    .and_then(|arg| match arg.value() {
                        ast::Value::String(s) => Some(s),
                        _ => None,
                    })
                    .map(|arguments| {
                        parse_selection_set(arguments).and_then(|fields| {
                            attach_input_value_set_to_field_arguments(fields, parent_id, field_id, state)
                        })
                    })
                    .transpose()?,
                fields: directive
                    .get_argument("fields")
                    .and_then(|arg| match arg.value() {
                        ast::Value::String(s) => Some(s),
                        _ => None,
                    })
                    .map(|fields| {
                        parse_selection_set(fields).and_then(|fields| attach_field_set(&fields, parent_id, state))
                    })
                    .transpose()?,
                node: directive
                    .get_argument("node")
                    .and_then(|arg| match arg.value() {
                        ast::Value::String(s) => Some(s),
                        _ => None,
                    })
                    .map(|fields| {
                        parse_selection_set(fields)
                            .and_then(|fields| attach_field_set(&fields, field_type.definition, state))
                    })
                    .transpose()?,
                metadata: directive
                    .get_argument("metadata")
                    .map(|metadata| state.insert_value(metadata.value(), None)),
            };
            state.authorized_directives.push(authorized_directive);
            let id = AuthorizedDirectiveId(state.authorized_directives.len() - 1);
            state.field_authorized_directives.push((field_id, id));
        }
    }

    Ok(())
}

fn ingest_definitions<'a>(document: &'a ast::TypeSystemDocument, state: &mut State<'a>) -> Result<(), DomainError> {
    for definition in document.definitions() {
        match definition {
            ast::Definition::SchemaExtension(_) | ast::Definition::Schema(_) | ast::Definition::Directive(_) => (),
            ast::Definition::TypeExtension(typedef) | ast::Definition::Type(typedef) => {
                let type_name = typedef.name();
                let type_name_id = state.insert_string(type_name);
                let description = typedef
                    .description()
                    .map(|description| state.insert_string(description.description().raw_str()));
                let composed_directives = collect_composed_directives(typedef.directives(), state);

                match typedef {
                    ast::TypeDefinition::Scalar(scalar) => {
                        let description = scalar
                            .description()
                            .map(|description| state.insert_string(description.description().raw_str()));

                        let scalar_id = ScalarId(state.scalars.push_return_idx(Scalar {
                            name: type_name_id,
                            composed_directives,
                            description,
                        }));
                        state.definition_names.insert(type_name, Definition::Scalar(scalar_id));
                    }
                    ast::TypeDefinition::Object(_) => {
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
                    ast::TypeDefinition::Interface(_) => {
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
                    ast::TypeDefinition::Union(_) => {
                        let union_id = UnionId(state.unions.push_return_idx(Union {
                            name: type_name_id,
                            members: Vec::new(),
                            composed_directives,
                            description,
                        }));
                        state.definition_names.insert(type_name, Definition::Union(union_id));
                    }
                    ast::TypeDefinition::Enum(enm) if type_name == JOIN_GRAPH_ENUM_NAME => {
                        ingest_join_graph_enum(enm, state)?;
                    }
                    ast::TypeDefinition::Enum(enm) => {
                        let enum_id = EnumId(state.enums.push_return_idx(Enum {
                            name: type_name_id,
                            values: NO_ENUM_VALUE,
                            composed_directives,
                            description,
                        }));
                        state.definition_names.insert(type_name, Definition::Enum(enum_id));

                        let values = {
                            let start = state.enum_values.len();

                            for value in enm.values() {
                                let composed_directives = collect_composed_directives(value.directives(), state);
                                let description = value
                                    .description()
                                    .map(|description| state.insert_string(description.description().raw_str()));

                                state
                                    .enum_values_map
                                    .insert((enum_id, value.value()), EnumValueId(state.enum_values.len()));

                                let value = state.insert_string(value.value());
                                state.enum_values.push(EnumValue {
                                    value,
                                    composed_directives,
                                    description,
                                });
                            }

                            (EnumValueId(start), state.enum_values.len() - start)
                        };

                        state.enums[enum_id.0].values = values;
                    }
                    ast::TypeDefinition::InputObject(_) => {
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

fn ingest_interface_fields<'a>(
    interface_id: InterfaceId,
    fields: impl Iterator<Item = ast::FieldDefinition<'a>>,
    state: &mut State<'a>,
) -> Result<(), DomainError> {
    let [mut start, mut end] = [None; 2];

    for field in fields {
        let field_id = ingest_field(Definition::Interface(interface_id), field, state)?;
        start = Some(start.unwrap_or(field_id));
        end = Some(field_id);
    }

    if let [Some(start), Some(end)] = [start, end] {
        state.interfaces[interface_id.0].fields = Range {
            start,
            end: FieldId(end.0 + 1),
        };
    };
    Ok(())
}

fn ingest_field<'a>(
    parent_id: Definition,
    ast_field: ast::FieldDefinition<'a>,
    state: &mut State<'a>,
) -> Result<FieldId, DomainError> {
    let field_name = ast_field.name();
    let r#type = state.field_type(ast_field.ty())?;
    let name = state.insert_string(field_name);
    let args_start = state.input_value_definitions.len();

    for arg in ast_field.arguments() {
        let description = arg
            .description()
            .map(|description| state.insert_string(description.description().raw_str()));
        let composed_directives = collect_composed_directives(arg.directives(), state);
        let name = state.insert_string(arg.name());
        let r#type = state.field_type(arg.ty())?;
        let default = arg
            .default_value()
            .map(|default| state.insert_value(default, r#type.definition.as_enum().copied()));

        state.input_value_definitions.push(InputValueDefinition {
            name,
            r#type,
            directives: composed_directives,
            description,
            default,
        });
    }

    let args_end = state.input_value_definitions.len();

    let resolvable_in = ast_field
        .directives()
        .filter(|dir| dir.name() == JOIN_FIELD_DIRECTIVE_NAME)
        // We implemented "overrides" by mistake, so we allow it for backwards compatibility..
        .filter(|dir| {
            dir.get_argument("overrides").is_none()
                && dir.get_argument(JOIN_FIELD_DIRECTIVE_OVERRIDE_ARGUMENT).is_none()
        })
        .filter(|dir| {
            !dir.get_argument("external")
                .map(|arg| match arg.value() {
                    ast::Value::Boolean(b) => b,
                    _ => false,
                })
                .unwrap_or_default()
        })
        .filter_map(|dir| dir.get_argument("graph"))
        .filter_map(|arg| match arg.value() {
            ast::Value::Enum(s) => Some(state.graph_sdl_names[s]),
            _ => None,
        })
        .collect();

    let overrides = ast_field
        .directives()
        .filter(|dir| dir.name() == JOIN_FIELD_DIRECTIVE_NAME)
        .filter_map(|dir| {
            dir.get_argument("graph")
                // We implemented "overrides" by mistake, so we allow it for backwards compatibility..
                .zip(
                    dir.get_argument("overrides")
                        .or(dir.get_argument(JOIN_FIELD_DIRECTIVE_OVERRIDE_ARGUMENT)),
                )
                .map(|(graph, overrides)| {
                    (
                        graph,
                        overrides,
                        dir.get_argument(JOIN_FIELD_DIRECTIVE_OVERRIDE_LABEL_ARGUMENT),
                    )
                })
        })
        .filter_map(
            |(graph, overrides, override_label)| match (graph.value(), overrides.value()) {
                (ast::Value::Enum(graph), ast::Value::String(overrides)) => {
                    Some(Override {
                        graph: state.graph_sdl_names.get(graph).copied().or_else(|| {
                            // Previously we used the subgraph name rather than the enum we overrides
                            // was specified.
                            let subgraph_name = state.insert_string(graph);
                            Some(SubgraphId(
                                state
                                    .subgraphs
                                    .iter()
                                    .position(|subgraph| subgraph.name == subgraph_name)?,
                            ))
                        })?,
                        label: override_label
                            .and_then(|arg| match arg.value() {
                                ast::Value::String(s) => s.parse().ok(),
                                _ => None,
                            })
                            .unwrap_or_default(),
                        from: state
                            .subgraphs
                            .iter()
                            .position(|subgraph| state.strings[subgraph.name.0] == overrides)
                            .map(SubgraphId)
                            .map(OverrideSource::Subgraph)
                            .unwrap_or_else(|| OverrideSource::Missing(state.insert_string(overrides))),
                    })
                }
                _ => None, // unreachable in valid schemas
            },
        )
        .collect();

    let composed_directives = collect_composed_directives(ast_field.directives(), state);
    let description = ast_field
        .description()
        .map(|description| state.insert_string(description.description().raw_str()));

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

    Ok(field_id)
}

fn ingest_union_members<'a>(
    union_id: UnionId,
    union: &ast::UnionDefinition<'a>,
    state: &mut State<'a>,
) -> Result<(), DomainError> {
    for member in union.members() {
        let Definition::Object(object_id) = state.definition_names[member.name()] else {
            return Err(DomainError("Non-object type in union members".to_owned()));
        };
        state.unions[union_id.0].members.push(object_id);
    }

    Ok(())
}

fn ingest_input_object<'a>(
    input_object_id: InputObjectId,
    input_object: &ast::InputObjectDefinition<'a>,
    state: &mut State<'a>,
) -> Result<(), DomainError> {
    let start = state.input_value_definitions.len();
    for field in input_object.fields() {
        state.input_values_map.insert(
            (input_object_id, field.name()),
            InputValueDefinitionId(state.input_value_definitions.len()),
        );
        let name = state.insert_string(field.name());
        let r#type = state.field_type(field.ty())?;
        let composed_directives = collect_composed_directives(field.directives(), state);
        let description = field
            .description()
            .map(|description| state.insert_string(description.description().raw_str()));
        let default = field
            .default_value()
            .map(|default| state.insert_value(default, r#type.definition.as_enum().copied()));

        state.input_value_definitions.push(InputValueDefinition {
            name,
            r#type,
            directives: composed_directives,
            description,
            default,
        });
    }
    let end = state.input_value_definitions.len();

    state.input_objects[input_object_id.0].fields = (InputValueDefinitionId(start), end - start);
    Ok(())
}

fn ingest_object_fields<'a>(
    object_id: ObjectId,
    fields: impl Iterator<Item = ast::FieldDefinition<'a>>,
    state: &mut State<'a>,
) -> Result<(), DomainError> {
    let [mut start, mut end] = [None; 2];

    for field in fields {
        let field_id = ingest_field(Definition::Object(object_id), field, state)?;
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

    if let [Some(start), Some(end)] = [start, end] {
        state.objects[object_id.0].fields = Range { start, end };
    };

    Ok(())
}

fn parse_selection_set(fields: &str) -> Result<executable_ast::ExecutableDocument, DomainError> {
    let fields = format!("{{ {fields} }}");

    cynic_parser::parse_executable_document(&fields)
        .map_err(|err| format!("Error parsing a selection from a federated directive: {err}"))
        .map_err(DomainError)
}

/// Attach a selection set defined in strings to a FederatedGraph, transforming the strings into
/// field ids.
fn attach_field_set(
    selection_set: &executable_ast::ExecutableDocument,
    target: Definition,
    state: &mut State<'_>,
) -> Result<FieldSet, DomainError> {
    let operation = selection_set
        .operations()
        .next()
        .expect("first operation is there by construction");

    attach_field_set_rec(operation.selection_set(), target, state)
}

fn attach_field_set_rec<'a>(
    selection_set: impl Iterator<Item = executable_ast::Selection<'a>>,
    target: Definition,
    state: &mut State<'_>,
) -> Result<FieldSet, DomainError> {
    selection_set
        .map(|selection| {
            let executable_ast::Selection::Field(ast_field) = &selection else {
                return Err(DomainError("Unsupported fragment spread in selection set".to_owned()));
            };
            let field: FieldId = *state.selection_map.get(&(target, ast_field.name())).ok_or_else(|| {
                DomainError(format!(
                    "Field '{}.{}' does not exist",
                    state.get_definition_name(target),
                    ast_field.name(),
                ))
            })?;
            let field_ty = state.fields[field.0].r#type.definition;
            let arguments = ast_field
                .arguments()
                .map(|argument| {
                    let name = state.insert_string(argument.name());
                    let (start, len) = state.fields[field.0].arguments;
                    let arguments = &state.input_value_definitions[start.0..start.0 + len];
                    let argument_id = arguments
                        .iter()
                        .position(|arg| arg.name == name)
                        .map(|idx| InputValueDefinitionId(start.0 + idx))
                        .expect("unknown argument");

                    let argument_type = state.input_value_definitions[argument_id.0]
                        .r#type
                        .definition
                        .as_enum()
                        .copied();

                    let converted_value = executable_value_to_type_system_value(argument.value());

                    let value = state.insert_value(converted_value, argument_type);

                    (argument_id, value)
                })
                .collect();

            Ok(FieldSetItem {
                field,
                arguments,
                subselection: attach_field_set_rec(ast_field.selection_set(), field_ty, state)?,
            })
        })
        .collect()
}

fn attach_input_value_set_to_field_arguments(
    selection_set: executable_ast::ExecutableDocument,
    parent: Definition,
    field_id: FieldId,
    state: &mut State<'_>,
) -> Result<InputValueDefinitionSet, DomainError> {
    let operation = selection_set
        .operations()
        .next()
        .expect("first operation is there by construction");

    attach_input_value_set_to_field_arguments_rec(operation.selection_set(), parent, field_id, state)
}

fn attach_input_value_set_to_field_arguments_rec<'a>(
    selection_set: impl Iterator<Item = executable_ast::Selection<'a>>,
    parent: Definition,
    field_id: FieldId,
    state: &mut State<'_>,
) -> Result<InputValueDefinitionSet, DomainError> {
    let (start, len) = state.fields[field_id.0].arguments;
    selection_set
        .map(|selection| {
            let executable_ast::Selection::Field(ast_arg) = selection else {
                return Err(DomainError("Unsupported fragment spread in selection set".to_owned()));
            };

            let arguments = &state.input_value_definitions[start.0..start.0 + len];
            let Some((i, arg)) = arguments
                .iter()
                .enumerate()
                .find(|(_, arg)| state.strings.get_index(arg.name.0).unwrap() == ast_arg.name())
            else {
                return Err(DomainError(format!(
                    "Argument '{}' does not exist for the field '{}.{}'",
                    ast_arg.name(),
                    state.get_definition_name(parent),
                    state.strings.get_index(state.fields[field_id.0].name.0).unwrap(),
                )));
            };

            let mut ast_subselection = ast_arg.selection_set().peekable();

            let subselection = if let Definition::InputObject(input_object_id) = arg.r#type.definition {
                if ast_subselection.peek().is_none() {
                    return Err(DomainError("InputObject must have a subselection".to_owned()));
                }
                attach_input_value_set_rec(ast_subselection, input_object_id, state)?
            } else if ast_subselection.peek().is_some() {
                return Err(DomainError("Only InputObject can have a subselection".to_owned()));
            } else {
                InputValueDefinitionSet::default()
            };

            Ok(InputValueDefinitionSetItem {
                input_value_definition: InputValueDefinitionId(start.0 + i),
                subselection,
            })
        })
        .collect()
}

fn attach_input_value_set_rec<'a>(
    selection_set: impl Iterator<Item = executable_ast::Selection<'a>>,
    input_object_id: InputObjectId,
    state: &mut State<'_>,
) -> Result<InputValueDefinitionSet, DomainError> {
    selection_set
        .map(|selection| {
            let executable_ast::Selection::Field(ast_field) = selection else {
                return Err(DomainError("Unsupported fragment spread in selection set".to_owned()));
            };
            let id = *state
                .input_values_map
                .get(&(input_object_id, ast_field.name()))
                .ok_or_else(|| {
                    DomainError(format!(
                        "Input field '{}.{}' does not exist",
                        state.get_definition_name(Definition::InputObject(input_object_id)),
                        ast_field.name(),
                    ))
                })?;

            let mut ast_subselection = ast_field.selection_set().peekable();

            let subselection = if let Definition::InputObject(input_object_id) =
                state.input_value_definitions[id.0].r#type.definition
            {
                if ast_subselection.peek().is_none() {
                    return Err(DomainError("InputObject must have a subselection".to_owned()));
                }
                attach_input_value_set_rec(ast_subselection, input_object_id, state)?
            } else if ast_subselection.peek().is_some() {
                return Err(DomainError("Only InputObject can have a subselection".to_owned()));
            } else {
                InputValueDefinitionSet::default()
            };

            Ok(InputValueDefinitionSetItem {
                input_value_definition: id,
                subselection,
            })
        })
        .collect()
}

fn ingest_join_graph_enum<'a>(enm: ast::EnumDefinition<'a>, state: &mut State<'a>) -> Result<(), DomainError> {
    for value in enm.values() {
        let sdl_name = value.value();
        let directive = value
            .directives()
            .find(|directive| directive.name() == JOIN_GRAPH_DIRECTIVE_NAME)
            .ok_or_else(|| DomainError("Missing @join__graph directive on join__Graph enum value.".to_owned()))?;
        let name = directive
            .get_argument("name")
            .ok_or_else(|| {
                DomainError(
                    "Missing `name` argument in `@join__graph` directive on `join__Graph` enum value.".to_owned(),
                )
            })
            .and_then(|arg| match arg.value() {
                ast::Value::String(s) => Ok(s),
                _ => Err(DomainError(
                    "Unexpected type for `name` argument in `@join__graph` directive on `join__Graph` enum value."
                        .to_owned(),
                )),
            })?;
        let url = directive
            .get_argument("url")
            .ok_or_else(|| {
                DomainError(
                    "Missing `url` argument in `@join__graph` directive on `join__Graph` enum value.".to_owned(),
                )
            })
            .and_then(|arg| match arg.value() {
                ast::Value::String(s) => Ok(s),
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

fn collect_composed_directives<'a>(
    directives: impl Iterator<Item = ast::Directive<'a>>,
    state: &mut State<'a>,
) -> Directives {
    let start = state.directives.len();

    for directive in directives
        .filter(|dir| dir.name() != JOIN_FIELD_DIRECTIVE_NAME)
        .filter(|dir| dir.name() != JOIN_TYPE_DIRECTIVE_NAME)
    {
        match directive.name() {
            "inaccessible" => state.directives.push(Directive::Inaccessible),
            "deprecated" => {
                let directive = Directive::Deprecated {
                    reason: directive.get_argument("reason").and_then(|value| match value.value() {
                        ast::Value::String(s) => Some(state.insert_string(s)),
                        _ => None,
                    }),
                };

                state.directives.push(directive)
            }
            "requiresScopes" => {
                let scopes: Option<Vec<Vec<String>>> = directive
                    .get_argument("scopes")
                    .and_then(|scopes| scopes.value().into_json())
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
                    .get_argument("policies")
                    .and_then(|policies| policies.value().into_json())
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
            "authenticated" => state.directives.push(Directive::Authenticated),
            // Added later after ingesting the graph.
            "authorized" => {}
            other => {
                let name = state.insert_string(other);
                let arguments = directive
                    .arguments()
                    .map(|arg| -> (StringId, Value) {
                        (state.insert_string(arg.name()), state.insert_value(arg.value(), None))
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
fn backwards_compatibility() {
    use expect_test::expect;

    let sdl = r###"
    directive @core(feature: String!) repeatable on SCHEMA

    directive @join__owner(graph: join__Graph!) on OBJECT

    directive @join__type(
        graph: join__Graph!
        key: String!
        resolvable: Boolean = true
    ) repeatable on OBJECT | INTERFACE

    directive @join__field(
        graph: join__Graph
        requires: String
        provides: String
    ) on FIELD_DEFINITION

    directive @join__graph(name: String!, url: String!) on ENUM_VALUE

    enum join__Graph {
        MANGROVE @join__graph(name: "mangrove", url: "http://example.com/mangrove")
        STEPPE @join__graph(name: "steppe", url: "http://example.com/steppe")
    }

    type Mammoth {
        tuskLength: Int
        weightGrams: Int @join__field(graph: mangrove, overrides: "steppe")
    }

    type Query {
        getMammoth: Mammoth @join__field(graph: mangrove, overrides: "steppe")
    }
    "###;

    let expected = expect![[r#"
    directive @core(feature: String!) repeatable on SCHEMA

    directive @join__owner(graph: join__Graph!) on OBJECT

    directive @join__type(
        graph: join__Graph!
        key: String!
        resolvable: Boolean = true
    ) repeatable on OBJECT | INTERFACE

    directive @join__field(
        graph: join__Graph
        requires: String
        provides: String
    ) on FIELD_DEFINITION

    directive @join__graph(name: String!, url: String!) on ENUM_VALUE

    enum join__Graph {
        MANGROVE @join__graph(name: "mangrove", url: "http://example.com/mangrove")
        STEPPE @join__graph(name: "steppe", url: "http://example.com/steppe")
    }

    type Mammoth {
        tuskLength: Int
        weightGrams: Int @join__field(graph: MANGROVE, override: "steppe")
    }

    type Query {
        getMammoth: Mammoth @join__field(graph: MANGROVE, override: "steppe")
    }
    "#]];
    let actual = crate::render_sdl::render_federated_sdl(&super::from_sdl(sdl).unwrap()).unwrap();

    expected.assert_eq(&actual);
}

#[cfg(test)]
#[test]
fn test_missing_type() {
    let sdl = r###"
    directive @core(feature: String!) repeatable on SCHEMA

    directive @join__owner(graph: join__Graph!) on OBJECT

    directive @join__type(
        graph: join__Graph!
        key: String!
        resolvable: Boolean = true
    ) repeatable on OBJECT | INTERFACE

    directive @join__field(
        graph: join__Graph
        requires: String
        provides: String
    ) on FIELD_DEFINITION

    directive @join__graph(name: String!, url: String!) on ENUM_VALUE

    enum join__Graph {
        MANGROVE @join__graph(name: "mangrove", url: "http://example.com/mangrove")
        STEPPE @join__graph(name: "steppe", url: "http://example.com/steppe")
    }

    type Query {
        getMammoth: Mammoth @join__field(graph: mangrove)
    }
    "###;
    let actual = super::from_sdl(sdl);
    assert!(actual.is_err());
}
