mod arguments;
mod directive;
mod value;

use self::{arguments::*, value::*};
use crate::{directives::*, federated_graph::*};
use cynic_parser::{
    common::WrappingType, executable as executable_ast, type_system as ast, values::ConstValue as ParserValue,
};
use directive::{
    collect_definition_directives, collect_enum_value_directives, collect_field_directives,
    collect_input_value_directives,
};
use indexmap::IndexSet;
use std::{collections::HashMap, error::Error as StdError, fmt, ops::Range};
use wrapping::Wrapping;

const JOIN_GRAPH_DIRECTIVE_NAME: &str = "join__graph";
const JOIN_GRAPH_ENUM_NAME: &str = "join__Graph";
const JOIN_FIELD_SET_SCALAR_NAME: &str = "join__FieldSet";

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

    graph: FederatedGraph,
    objects: Vec<Object>,
    interfaces: Vec<Interface>,
    fields: Vec<Field>,

    input_value_definitions: Vec<InputValueDefinition>,

    unions: Vec<Union>,
    input_objects: Vec<InputObject>,

    strings: IndexSet<String>,
    query_type_name: Option<String>,
    mutation_type_name: Option<String>,
    subscription_type_name: Option<String>,

    definition_names: HashMap<&'a str, Definition>,
    selection_map: HashMap<(Definition, &'a str), FieldId>,
    input_values_map: HashMap<(InputObjectId, &'a str), InputValueDefinitionId>,
    enum_values_map: HashMap<(TypeDefinitionId, &'a str), EnumValueId>,

    /// The key is the name of the graph in the join__Graph enum.
    graph_by_enum_str: HashMap<&'a str, SubgraphId>,
    graph_by_name: HashMap<&'a str, SubgraphId>,

    type_wrappers: Vec<WrappingType>,
}

macro_rules! id_newtypes {
    ($($storage:ident [ $name:ident ] -> $out:ident,)*) => {
        $(
            impl std::ops::Index<$name> for State<'_> {
                type Output = $out;

                fn index(&self, index: $name) -> &$out {
                    &self.$storage[usize::from(index)]
                }
            }

            impl std::ops::IndexMut<$name> for State<'_> {
                fn index_mut(&mut self, index: $name) -> &mut $out {
                    &mut self.$storage[usize::from(index)]
                }
            }
        )*
    }
}

id_newtypes! {
    fields[FieldId] -> Field,
    input_objects[InputObjectId] -> InputObject,
    input_value_definitions[InputValueDefinitionId] -> InputValueDefinition,
    interfaces[InterfaceId] -> Interface,
    objects[ObjectId] -> Object,
    subgraphs[SubgraphId] -> Subgraph,
    unions[UnionId] -> Union,
}

impl std::ops::Index<StringId> for State<'_> {
    type Output = str;

    fn index(&self, index: StringId) -> &Self::Output {
        &self.strings[usize::from(index)]
    }
}

impl<'a> State<'a> {
    fn field_type(&mut self, field_type: ast::Type<'a>) -> Result<Type, DomainError> {
        self.field_type_from_name_and_wrapping(field_type.name(), field_type.wrappers())
    }

    fn field_type_from_str(&mut self, ty: &str) -> Result<Type, DomainError> {
        let mut wrappers = Vec::new();
        let mut chars = ty.chars().rev();

        let mut start = 0;
        let mut end = ty.len();
        loop {
            match chars.next() {
                Some('!') => {
                    wrappers.push(WrappingType::NonNull);
                }
                Some(']') => {
                    wrappers.push(WrappingType::List);
                    start += 1;
                }
                _ => break,
            }
            end -= 1;
        }
        self.field_type_from_name_and_wrapping(&ty[start..end], wrappers)
    }

    fn field_type_from_name_and_wrapping(
        &mut self,
        name: &str,
        wrappers: impl IntoIterator<Item = WrappingType>,
    ) -> Result<Type, DomainError> {
        use cynic_parser::common::WrappingType;

        self.type_wrappers.clear();
        self.type_wrappers.extend(wrappers);
        self.type_wrappers.reverse();

        let mut wrappers = self.type_wrappers.iter().peekable();

        let mut wrapping = match wrappers.peek() {
            Some(WrappingType::NonNull) => {
                wrappers.next();
                wrapping::Wrapping::new(true)
            }
            _ => wrapping::Wrapping::new(false),
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
            .get(name)
            .ok_or_else(|| DomainError(format!("Unknown type '{}'", name)))?;

        Ok(Type { definition, wrapping })
    }

    fn insert_string(&mut self, s: &str) -> StringId {
        if let Some(idx) = self.strings.get_index_of(s) {
            return StringId::from(idx);
        }

        StringId::from(self.strings.insert_full(s.to_owned()).0)
    }

    fn insert_value(&mut self, node: ParserValue<'_>, expected_enum_type: Option<TypeDefinitionId>) -> Value {
        match node {
            ParserValue::Null(_) => Value::Null,
            ParserValue::Int(n) => Value::Int(n.as_i64()),
            ParserValue::Float(n) => Value::Float(n.as_f64()),
            ParserValue::String(s) => Value::String(self.insert_string(s.value())),
            ParserValue::Boolean(b) => Value::Boolean(b.value()),
            ParserValue::Enum(enm) => expected_enum_type
                .and_then(|enum_id| {
                    let enum_value_id = self.enum_values_map.get(&(enum_id, enm.name()))?;
                    Some(Value::EnumValue(*enum_value_id))
                })
                .unwrap_or(Value::UnboundEnumValue(self.insert_string(enm.name()))),
            ParserValue::List(list) => Value::List(
                list.items()
                    .map(|value| self.insert_value(value, expected_enum_type))
                    .collect(),
            ),
            ParserValue::Object(obj) => Value::Object(
                obj.fields()
                    .map(|field| {
                        (
                            self.insert_string(field.name()),
                            self.insert_value(field.value(), expected_enum_type),
                        )
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            ),
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
            Definition::Object(object_id) => {
                self.graph
                    .view(self.objects[usize::from(object_id)].type_definition_id)
                    .name
            }
            Definition::Interface(interface_id) => {
                self.graph
                    .view(self.interfaces[usize::from(interface_id)].type_definition_id)
                    .name
            }
            Definition::Scalar(scalar_id) => self.graph[scalar_id].name,
            Definition::Enum(enum_id) => self.graph[enum_id].name,
            Definition::Union(union_id) => self.unions[usize::from(union_id)].name,
            Definition::InputObject(input_object_id) => self.input_objects[usize::from(input_object_id)].name,
        };
        &self.strings[usize::from(name)]
    }
}

pub fn from_sdl(sdl: &str) -> Result<FederatedGraph, DomainError> {
    let mut state = State::default();
    state.graph.strings.clear();
    state.graph.objects.clear();
    state.graph.type_definitions.clear();
    state.graph.fields.clear();

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

        let object_id = ObjectId::from(state.objects.len());
        let query_string_id = state.insert_string(query_type_name);

        state
            .definition_names
            .insert(query_type_name, Definition::Object(object_id));

        let type_definition_id = state.graph.push_type_definition(TypeDefinitionRecord {
            name: query_string_id,
            description: None,
            directives: Vec::new(),
            kind: TypeDefinitionKind::Object,
        });

        state.objects.push(Object {
            type_definition_id,
            implements_interfaces: Vec::new(),
            fields: NO_FIELDS,
        });

        ingest_object_fields(object_id, std::iter::empty(), &mut state)?;
    }

    ingest_fields(&parsed, &mut state)?;

    // This needs to happen after all fields have been ingested, in order to attach selection sets.
    ingest_directives_after_graph(&parsed, &mut state)?;

    let mut graph = FederatedGraph {
        type_definitions: std::mem::take(&mut state.graph.type_definitions),
        root_operation_types: state.root_operation_types()?,
        subgraphs: state.subgraphs,
        objects: state.objects,
        interfaces: state.interfaces,
        fields: state.fields,
        enum_values: std::mem::take(&mut state.graph.enum_values),
        unions: state.unions,
        input_objects: state.input_objects,
        strings: state.strings.into_iter().collect(),
        input_value_definitions: state.input_value_definitions,
    };

    graph.enum_values.sort_unstable_by_key(|v| v.enum_id);

    Ok(graph)
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
                ast::TypeDefinition::Interface(interface) => {
                    let Definition::Interface(interface_id) = state.definition_names[typedef.name()] else {
                        return Err(DomainError(
                            "Broken invariant: interface id behind interface name.".to_owned(),
                        ));
                    };
                    ingest_interface_interfaces(interface_id, interface, state)?;
                    ingest_interface_fields(interface_id, interface.fields(), state)?;
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
    state.interfaces[usize::from(interface_id)].implements_interfaces = interface
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
    state.objects[usize::from(object_id)].implements_interfaces = object
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

fn ingest_directives_after_graph<'a>(
    parsed: &'a ast::TypeSystemDocument,
    state: &mut State<'a>,
) -> Result<(), DomainError> {
    for definition in parsed.definitions() {
        let (ast::Definition::Type(typedef) | ast::Definition::TypeExtension(typedef)) = definition else {
            continue;
        };

        // Some definitions such as join__Graph or join__FieldSet
        let Some(definition_id) = state.definition_names.get(typedef.name()).copied() else {
            continue;
        };
        let directives = collect_definition_directives(definition_id, typedef.directives(), state)?;

        match definition_id {
            Definition::Scalar(id) => state.graph[id].directives = directives,
            Definition::Object(id) => {
                let id = state[id].type_definition_id;
                state.graph[id].directives = directives
            }
            Definition::Interface(id) => {
                let id = state[id].type_definition_id;
                state.graph[id].directives = directives;
            }
            Definition::Union(id) => state[id].directives = directives,
            Definition::Enum(id) => state.graph[id].directives = directives,
            Definition::InputObject(id) => state[id].directives = directives,
        }

        let fields = match typedef {
            ast::TypeDefinition::Object(object) => Some(object.fields()),
            ast::TypeDefinition::Interface(iface) => Some(iface.fields()),
            _ => None,
        };
        if let Some(fields) = fields {
            for field in fields {
                let field_id = state.selection_map[&(definition_id, field.name())];
                state[field_id].directives =
                    collect_field_directives(definition_id, field_id, field.directives(), state)?;
            }
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

                match typedef {
                    ast::TypeDefinition::Enum(enm) if type_name == JOIN_GRAPH_ENUM_NAME => {
                        ingest_join_graph_enum(enm, state)?;
                        continue;
                    }
                    ast::TypeDefinition::Scalar(_) if type_name == JOIN_FIELD_SET_SCALAR_NAME => {
                        continue;
                    }
                    _ => (),
                }

                let type_name_id = state.insert_string(type_name);
                let description = typedef
                    .description()
                    .map(|description| state.insert_string(&description.to_cow()));

                let type_definition_id = state.graph.push_type_definition(TypeDefinitionRecord {
                    name: type_name_id,
                    description,
                    directives: Vec::new(),
                    kind: match typedef {
                        ast::TypeDefinition::Scalar(_) => TypeDefinitionKind::Scalar,
                        ast::TypeDefinition::Object(_) => TypeDefinitionKind::Object,
                        ast::TypeDefinition::Interface(_) => TypeDefinitionKind::Interface,
                        ast::TypeDefinition::Union(_) => TypeDefinitionKind::Union,
                        ast::TypeDefinition::Enum(_) => TypeDefinitionKind::Enum,
                        ast::TypeDefinition::InputObject(_) => TypeDefinitionKind::InputObject,
                    },
                });

                match typedef {
                    ast::TypeDefinition::Scalar(_) => {
                        state
                            .definition_names
                            .insert(type_name, Definition::Scalar(type_definition_id));
                    }
                    ast::TypeDefinition::Object(_) => {
                        let object_id = ObjectId::from(state.objects.push_return_idx(Object {
                            type_definition_id,
                            implements_interfaces: Vec::new(),
                            fields: NO_FIELDS,
                        }));

                        state.definition_names.insert(type_name, Definition::Object(object_id));
                    }
                    ast::TypeDefinition::Interface(_) => {
                        let interface_id = InterfaceId::from(state.interfaces.push_return_idx(Interface {
                            type_definition_id,
                            implements_interfaces: Vec::new(),
                            fields: NO_FIELDS,
                        }));
                        state
                            .definition_names
                            .insert(type_name, Definition::Interface(interface_id));
                    }
                    ast::TypeDefinition::Union(_) => {
                        let union_id = UnionId::from(state.unions.push_return_idx(Union {
                            name: type_name_id,
                            members: Vec::new(),
                            description,
                            directives: Vec::new(),
                        }));
                        state.definition_names.insert(type_name, Definition::Union(union_id));
                    }
                    ast::TypeDefinition::Enum(enm) => {
                        state
                            .definition_names
                            .insert(type_name, Definition::Enum(type_definition_id));

                        for value in enm.values() {
                            let description = value
                                .description()
                                .map(|description| state.insert_string(&description.to_cow()));

                            let directives = collect_enum_value_directives(value.directives(), state)?;
                            let value_string_id = state.insert_string(value.value());
                            let id = state.graph.push_enum_value(EnumValueRecord {
                                enum_id: type_definition_id,
                                value: value_string_id,
                                directives,
                                description,
                            });

                            state.enum_values_map.insert((type_definition_id, value.value()), id);
                        }
                    }
                    ast::TypeDefinition::InputObject(_) => {
                        let input_object_id = InputObjectId::from(state.input_objects.push_return_idx(InputObject {
                            name: type_name_id,
                            fields: NO_INPUT_VALUE_DEFINITION,
                            directives: Vec::new(),
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
        let id = state.graph.push_type_definition(TypeDefinitionRecord {
            name,
            directives: Vec::new(),
            description: None,
            kind: TypeDefinitionKind::Scalar,
        });
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
        let field_id = ingest_field(EntityDefinitionId::Interface(interface_id), field, state)?;
        start = Some(start.unwrap_or(field_id));
        end = Some(field_id);
    }

    if let [Some(start), Some(end)] = [start, end] {
        state.interfaces[usize::from(interface_id)].fields = Range {
            start,
            end: FieldId::from(usize::from(end) + 1),
        };
    };
    Ok(())
}

fn ingest_field<'a>(
    parent_entity_id: EntityDefinitionId,
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
            .map(|description| state.insert_string(&description.to_cow()));
        let directives = collect_input_value_directives(arg.directives(), state)?;
        let name = state.insert_string(arg.name());
        let r#type = state.field_type(arg.ty())?;
        let default = arg
            .default_value()
            .map(|default| state.insert_value(default, r#type.definition.as_enum()));

        state.input_value_definitions.push(InputValueDefinition {
            name,
            r#type,
            directives,
            description,
            default,
        });
    }

    let args_end = state.input_value_definitions.len();

    let description = ast_field
        .description()
        .map(|description| state.insert_string(&description.to_cow()));

    let field_id = FieldId::from(state.fields.push_return_idx(Field {
        name,
        r#type,
        parent_entity_id,
        arguments: (InputValueDefinitionId::from(args_start), args_end - args_start),
        description,
        // Added at the end.
        directives: Vec::new(),
    }));

    state
        .selection_map
        .insert((parent_entity_id.into(), field_name), field_id);

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
        state.unions[usize::from(union_id)].members.push(object_id);
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
            InputValueDefinitionId::from(state.input_value_definitions.len()),
        );
        let name = state.insert_string(field.name());
        let r#type = state.field_type(field.ty())?;
        let directives = collect_input_value_directives(field.directives(), state)?;
        let description = field
            .description()
            .map(|description| state.insert_string(&description.to_cow()));
        let default = field
            .default_value()
            .map(|default| state.insert_value(default, r#type.definition.as_enum()));

        state.input_value_definitions.push(InputValueDefinition {
            name,
            r#type,
            directives,
            description,
            default,
        });
    }
    let end = state.input_value_definitions.len();

    state.input_objects[usize::from(input_object_id)].fields = (InputValueDefinitionId::from(start), end - start);
    Ok(())
}

fn ingest_object_fields<'a>(
    object_id: ObjectId,
    fields: impl Iterator<Item = ast::FieldDefinition<'a>>,
    state: &mut State<'a>,
) -> Result<(), DomainError> {
    let start = state.fields.len();
    for field in fields {
        ingest_field(EntityDefinitionId::Object(object_id), field, state)?;
    }

    // When we encounter the root query type, we need to make space at the end of the fields for __type and __schema.
    if object_id
        == state
            .root_operation_types()
            .expect("root operation types to be defined at this point")
            .query
    {
        for name in ["__schema", "__type"].map(|name| state.insert_string(name)) {
            state.fields.push(Field {
                name,
                r#type: Type {
                    wrapping: Wrapping::new(false),
                    definition: Definition::Object(object_id),
                },
                parent_entity_id: EntityDefinitionId::Object(object_id),
                arguments: NO_INPUT_VALUE_DEFINITION,
                description: None,
                // Added later
                directives: Vec::new(),
            });
        }
    }

    state.objects[usize::from(object_id)].fields = Range {
        start: FieldId::from(start),
        end: FieldId::from(state.fields.len()),
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
fn attach_selection_set(
    selection_set: &executable_ast::ExecutableDocument,
    target: Definition,
    state: &mut State<'_>,
) -> Result<SelectionSet, DomainError> {
    let operation = selection_set
        .operations()
        .next()
        .expect("first operation is there by construction");

    attach_selection_set_rec(operation.selection_set(), target, state)
}

fn attach_selection_set_rec<'a>(
    selection_set: impl Iterator<Item = executable_ast::Selection<'a>>,
    target: Definition,
    state: &mut State<'_>,
) -> Result<SelectionSet, DomainError> {
    selection_set
        .map(|selection| match selection {
            executable_ast::Selection::Field(ast_field) => attach_selection_field(ast_field, target, state),
            executable_ast::Selection::InlineFragment(inline_fragment) => {
                attach_inline_fragment(inline_fragment, state)
            }
            executable_ast::Selection::FragmentSpread(_) => {
                Err(DomainError("Unsupported fragment spread in selection set".to_owned()))
            }
        })
        .collect()
}

fn attach_selection_field(
    ast_field: executable_ast::FieldSelection<'_>,
    target: Definition,
    state: &mut State<'_>,
) -> Result<Selection, DomainError> {
    let field_id: FieldId = *state.selection_map.get(&(target, ast_field.name())).ok_or_else(|| {
        DomainError(format!(
            "Field '{}.{}' does not exist",
            state.get_definition_name(target),
            ast_field.name(),
        ))
    })?;
    let field_ty = state.fields[usize::from(field_id)].r#type.definition;
    let arguments = ast_field
        .arguments()
        .map(|argument| {
            let name = state.insert_string(argument.name());
            let (start, len) = state.fields[usize::from(field_id)].arguments;
            let arguments = &state.input_value_definitions[usize::from(start)..usize::from(start) + len];
            let argument_id = arguments
                .iter()
                .position(|arg| arg.name == name)
                .map(|idx| InputValueDefinitionId::from(usize::from(start) + idx))
                .expect("unknown argument");

            let argument_type = state.input_value_definitions[usize::from(argument_id)]
                .r#type
                .definition
                .as_enum();

            let const_value = argument
                .value()
                .try_into()
                .map_err(|_| DomainError("FieldSets cant contain variables".into()))?;

            let value = state.insert_value(const_value, argument_type);

            Ok((argument_id, value))
        })
        .collect::<Result<_, _>>()?;

    Ok(Selection::Field(FieldSelection {
        field_id,
        arguments,
        subselection: attach_selection_set_rec(ast_field.selection_set(), field_ty, state)?,
    }))
}

fn attach_inline_fragment(
    inline_fragment: executable_ast::InlineFragment<'_>,
    state: &mut State<'_>,
) -> Result<Selection, DomainError> {
    let on: Definition = match inline_fragment.type_condition() {
        Some(type_name) => *state
            .definition_names
            .get(type_name)
            .ok_or_else(|| DomainError(format!("Type '{}' in type condition does not exist", type_name)))?,
        None => {
            return Err(DomainError(
                "Fragments without type condition are not supported".to_owned(),
            ))
        }
    };

    let subselection = attach_selection_set_rec(inline_fragment.selection_set(), on, state)?;

    Ok(Selection::InlineFragment { on, subselection })
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
    let (start, len) = state.fields[usize::from(field_id)].arguments;
    selection_set
        .map(|selection| {
            let executable_ast::Selection::Field(ast_arg) = selection else {
                return Err(DomainError("Unsupported fragment spread in selection set".to_owned()));
            };

            let arguments = &state.input_value_definitions[usize::from(start)..usize::from(start) + len];
            let Some((i, arg)) = arguments
                .iter()
                .enumerate()
                .find(|(_, arg)| state.strings.get_index(usize::from(arg.name)).unwrap() == ast_arg.name())
            else {
                return Err(DomainError(format!(
                    "Argument '{}' does not exist for the field '{}.{}'",
                    ast_arg.name(),
                    state.get_definition_name(parent),
                    state
                        .strings
                        .get_index(usize::from(state.fields[usize::from(field_id)].name))
                        .unwrap(),
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
                input_value_definition: InputValueDefinitionId::from(usize::from(start) + i),
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
                state.input_value_definitions[usize::from(id)].r#type.definition
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
            .and_then(|arg| match arg {
                ParserValue::String(s) => Ok(s),
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
            .and_then(|arg| match arg {
                ParserValue::String(s) => Ok(s),
                _ => Err(DomainError(
                    "Unexpected type for `url` argument in `@join__graph` directive on `join__Graph` enum value."
                        .to_owned(),
                )),
            })?;

        let subgraph_name = state.insert_string(name.value());
        let url = state.insert_string(url.value());
        let id = SubgraphId::from(state.subgraphs.push_return_idx(Subgraph {
            name: subgraph_name,
            url,
        }));
        state.graph_by_enum_str.insert(sdl_name, id);
        state.graph_by_name.insert(name.value(), id);
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
            .any(|f| usize::from(f.name) == field_name));
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
            .any(|f| usize::from(f.name) == field_name));
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
            .any(|f| usize::from(f.name) == field_name));
    }
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

#[cfg(test)]
#[test]
fn test_join_field_type() {
    use expect_test::expect;

    let sdl = r###"
    schema
      @link(url: "https://specs.apollo.dev/link/v1.0")
      @link(url: "https://specs.apollo.dev/join/v0.3", for: EXECUTION) {
      query: Query
    }

    directive @join__enumValue(graph: join__Graph!) repeatable on ENUM_VALUE

    directive @join__field(
      graph: join__Graph
      requires: join__FieldSet
      provides: join__FieldSet
      type: String
      external: Boolean
      override: String
      usedOverridden: Boolean
    ) repeatable on FIELD_DEFINITION | INPUT_FIELD_DEFINITION

    directive @join__graph(name: String!, url: String!) on ENUM_VALUE

    directive @join__implements(
      graph: join__Graph!
      interface: String!
    ) repeatable on OBJECT | INTERFACE

    directive @join__type(
      graph: join__Graph!
      key: join__FieldSet
      extension: Boolean! = false
      resolvable: Boolean! = true
      isInterfaceObject: Boolean! = false
    ) repeatable on OBJECT | INTERFACE | UNION | ENUM | INPUT_OBJECT | SCALAR

    directive @join__unionMember(
      graph: join__Graph!
      member: String!
    ) repeatable on UNION

    directive @link(
      url: String
      as: String
      for: link__Purpose
      import: [link__Import]
    ) repeatable on SCHEMA

    union Account
      @join__type(graph: B)
      @join__unionMember(graph: B, member: "User")
      @join__unionMember(graph: B, member: "Admin") =
      | User
      | Admin

    type Admin @join__type(graph: B) {
      id: ID
      name: String
      similarAccounts: [Account!]!
    }

    scalar join__FieldSet

    enum join__Graph {
      A @join__graph(name: "a", url: "http://localhost:4200/child-type-mismatch/a")
      B @join__graph(name: "b", url: "http://localhost:4200/child-type-mismatch/b")
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

    type Query @join__type(graph: A) @join__type(graph: B) {
      users: [User!]! @join__field(graph: A)
      accounts: [Account!]! @join__field(graph: B)
    }

    type User @join__type(graph: A) @join__type(graph: B, key: "id") {
      id: ID @join__field(graph: A, type: "ID") @join__field(graph: B, type: "ID!")
      name: String @join__field(graph: B)
      similarAccounts: [Account!]! @join__field(graph: B)
    }
    "###;

    let expected = expect![[r#"
        directive @core(feature: String!) repeatable on SCHEMA

        directive @join__owner(graph: join__Graph!) on OBJECT

        directive @join__type(
            graph: join__Graph!
            key: join__FieldSet
            resolvable: Boolean = true
        ) repeatable on OBJECT | INTERFACE

        directive @join__field(
            graph: join__Graph
            requires: join__FieldSet
            provides: join__FieldSet
        ) on FIELD_DEFINITION

        directive @join__graph(name: String!, url: String!) on ENUM_VALUE

        directive @join__implements(graph: join__Graph!, interface: String!) repeatable on OBJECT | INTERFACE

        directive @join__unionMember(graph: join__Graph!, member: String!) repeatable on UNION

        scalar join__FieldSet

        enum join__Graph {
            A @join__graph(name: "a", url: "http://localhost:4200/child-type-mismatch/a")
            B @join__graph(name: "b", url: "http://localhost:4200/child-type-mismatch/b")
        }

        scalar link__Import

        type Admin
            @join__type(graph: B)
        {
            id: ID
            name: String
            similarAccounts: [Account!]!
        }

        type Query
            @join__type(graph: A)
            @join__type(graph: B)
        {
            users: [User!]! @join__field(graph: A)
            accounts: [Account!]! @join__field(graph: B)
        }

        type User
            @join__type(graph: A)
            @join__type(graph: B, key: "id")
        {
            id: ID @join__field(graph: A, type: "ID") @join__field(graph: B, type: "ID!")
            name: String @join__field(graph: B)
            similarAccounts: [Account!]! @join__field(graph: B)
        }

        enum link__Purpose
        {
            """
            `SECURITY` features provide metadata necessary to securely resolve fields.
            """
            SECURITY
            """
            `EXECUTION` features provide metadata necessary for operation execution.
            """
            EXECUTION
        }

        union Account
            @join__type(graph: B)
            @join__unionMember(graph: B, member: "User")
            @join__unionMember(graph: B, member: "Admin")
         = User | Admin
    "#]];

    let actual = crate::render_sdl::render_federated_sdl(&super::from_sdl(sdl).unwrap()).unwrap();

    expected.assert_eq(&actual);
}
