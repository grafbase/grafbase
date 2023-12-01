use std::collections::HashMap;

// All of that should be in federated_graph actually.
use super::*;
use crate::introspection::Introspection;

#[allow(clippy::panic)]
impl From<federated_graph::FederatedGraph> for Schema {
    fn from(graph: federated_graph::FederatedGraph) -> Self {
        let federated_graph::FederatedGraph::V1(graph) = graph;
        let mut schema = Schema {
            description: None,
            data_sources: (0..graph.subgraphs.len())
                .map(|i| DataSource::Subgraph(SubgraphId::from(i)))
                .collect(),
            subgraphs: graph.subgraphs.into_iter().map(Into::into).collect(),
            root_operation_types: RootOperationTypes {
                query: graph.root_operation_types.query.into(),
                mutation: graph.root_operation_types.mutation.map(Into::into),
                subscription: graph.root_operation_types.subscription.map(Into::into),
            },
            objects: graph.objects.into_iter().map(Into::into).collect(),
            object_fields: graph.object_fields.into_iter().map(Into::into).collect(),
            fields: vec![],
            types: graph.field_types.into_iter().map(Into::into).collect(),
            interfaces: graph.interfaces.into_iter().map(Into::into).collect(),
            interface_fields: graph.interface_fields.into_iter().map(Into::into).collect(),
            enums: graph.enums.into_iter().map(Into::into).collect(),
            unions: graph.unions.into_iter().map(Into::into).collect(),
            scalars: vec![],
            input_objects: vec![],
            strings: graph.strings,
            resolvers: vec![],
            definitions: vec![],
            input_values: vec![],
        };

        let root_fields = {
            let mut root_fields = vec![];
            let walker = schema.default_walker();
            for field in walker.walk(schema.root_operation_types.query).fields() {
                root_fields.push(field.id);
            }
            if let Some(mutation) = schema.root_operation_types.mutation {
                for field in walker.walk(mutation).fields() {
                    root_fields.push(field.id);
                }
            }
            if let Some(subscription) = schema.root_operation_types.subscription {
                for field in walker.walk(subscription).fields() {
                    root_fields.push(field.id);
                }
            }
            root_fields.sort_unstable();
            root_fields
        };

        // Yeah it's ugly, conversion should be cleaned up once we got it working I guess.
        // -- FIELDS & RESOLVERS --
        // 1. The federated graph uses "resolvable_in" whenever a field is present in a subgraph.
        //    But for resolvers we only want the "entrypoints", so root fields and later the `@key`
        //    for federation entities.
        // 2. Field arguments are converted to input values. That's how the GraphQL spec defines
        //    them and having an id allows data sources to rename those more easily.
        let mut resolvers = HashMap::<Resolver, ResolverId>::new();
        for (i, field) in graph.fields.into_iter().enumerate() {
            let field_id = FieldId::from(i);
            let mut field_resolvers = vec![];
            let mut field_requires = field
                .requires
                .into_iter()
                .map(|federated_graph::FieldRequires { subgraph_id, fields }| (subgraph_id, fields))
                .collect::<HashMap<_, _>>();
            if let Some(subgraph_id) = field.resolvable_in {
                if root_fields.binary_search(&field_id).is_ok() {
                    let n = resolvers.len();
                    let resolver_id = *resolvers
                        .entry(Resolver::Subgraph(SubgraphResolver {
                            subgraph_id: subgraph_id.into(),
                        }))
                        .or_insert_with(|| ResolverId::from(n));
                    let requires = field_requires.remove(&subgraph_id).unwrap_or_default();
                    field_resolvers.push(FieldResolver {
                        resolver_id,
                        requires: requires.into_iter().map(Into::into).collect(),
                    });
                }
            }
            let field = Field {
                name: field.name.into(),
                description: None,
                type_id: field.field_type_id.into(),
                resolvers: field_resolvers,
                provides: field.provides.into_iter().map(Into::into).collect(),
                arguments: {
                    field
                        .arguments
                        .into_iter()
                        .map(|argument| {
                            let input_value = InputValue {
                                name: argument.name.into(),
                                description: None,
                                type_id: argument.type_id.into(),
                                default_value: None,
                            };
                            schema.input_values.push(input_value);
                            InputValueId::from(schema.input_values.len() - 1)
                        })
                        .collect()
                },
                composed_directives: field.composed_directives.into_iter().map(Into::into).collect(),
                is_deprecated: false,
                deprecation_reason: None,
            };
            schema.fields.push(field);
        }
        let mut resolvers = resolvers.into_iter().collect::<Vec<_>>();
        resolvers.sort_unstable_by_key(|(_, resolver_id)| *resolver_id);
        schema.resolvers = resolvers.into_iter().map(|(resolver, _)| resolver).collect();

        // -- INPUT OBJECTS --
        // Separating the input fields into a separate input_value vec with an id. This additional
        // indirection allows data sources to rename fields more easily.
        for input_object in graph.input_objects {
            let input_object = InputObject {
                name: input_object.name.into(),
                description: None,
                input_fields: {
                    input_object
                        .fields
                        .into_iter()
                        .map(|field| {
                            let input_value = InputValue {
                                name: field.name.into(),
                                description: None,
                                type_id: field.field_type_id.into(),
                                default_value: None,
                            };
                            schema.input_values.push(input_value);
                            InputValueId::from(schema.input_values.len() - 1)
                        })
                        .collect()
                },
                composed_directives: input_object.composed_directives.into_iter().map(Into::into).collect(),
            };
            schema.input_objects.push(input_object);
        }

        // -- INTERFACES --
        // Adding all implementations of an interface, used during introspection.
        for object_id in (0..schema.objects.len()).map(ObjectId::from) {
            for interface_id in schema[object_id].interfaces.clone() {
                schema[interface_id].possible_types.push(object_id);
            }
        }

        // -- SCALARS --
        schema.scalars = graph
            .scalars
            .into_iter()
            .map(|scalar| {
                let name = StringId::from(scalar.name);
                Scalar {
                    name,
                    data_type: DataType::from_scalar_name(&schema[name]),
                    description: None,
                    specified_by_url: None,
                    composed_directives: scalar.composed_directives.into_iter().map(Into::into).collect(),
                }
            })
            .collect();

        // -- INTROSPECTION --
        Introspection::finalize_schema(schema)
    }
}

impl From<federated_graph::Subgraph> for Subgraph {
    fn from(subgraph: federated_graph::Subgraph) -> Self {
        Subgraph {
            name: subgraph.name.into(),
            url: subgraph.url.into(),
        }
    }
}

impl From<federated_graph::Object> for Object {
    fn from(object: federated_graph::Object) -> Self {
        Object {
            name: object.name.into(),
            description: None,
            interfaces: object.implements_interfaces.into_iter().map(Into::into).collect(),
            resolvable_keys: object.resolvable_keys.into_iter().map(Into::into).collect(),
            composed_directives: object.composed_directives.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<federated_graph::Directive> for Directive {
    fn from(directive: federated_graph::Directive) -> Self {
        Directive {
            name: directive.name.into(),
            arguments: directive
                .arguments
                .into_iter()
                .map(|(id, value)| (id.into(), value.into()))
                .collect(),
        }
    }
}

impl From<federated_graph::ObjectField> for ObjectField {
    fn from(object_field: federated_graph::ObjectField) -> Self {
        ObjectField {
            object_id: object_field.object_id.into(),
            field_id: object_field.field_id.into(),
        }
    }
}

impl From<federated_graph::FieldProvides> for FieldProvides {
    fn from(provides: federated_graph::FieldProvides) -> Self {
        FieldProvides {
            data_source_id: DataSourceId::from_subgraph_id(provides.subgraph_id),
            fields: provides.fields.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<federated_graph::Value> for Value {
    fn from(value: federated_graph::Value) -> Self {
        match value {
            federated_graph::Value::String(s) => Value::String(s.into()),
            federated_graph::Value::Int(i) => Value::Int(i),
            federated_graph::Value::Float(f) => Value::Float(f.into()),
            federated_graph::Value::Boolean(b) => Value::Boolean(b),
            federated_graph::Value::EnumValue(s) => Value::EnumValue(s.into()),
            federated_graph::Value::Object(fields) => Value::Object(
                fields
                    .into_iter()
                    .map(|(id, value)| (id.into(), value.into()))
                    .collect(),
            ),
            federated_graph::Value::List(l) => Value::List(l.into_iter().map(Into::into).collect()),
        }
    }
}

impl From<federated_graph::Definition> for Definition {
    fn from(definition: federated_graph::Definition) -> Self {
        match definition {
            federated_graph::Definition::Scalar(id) => Definition::Scalar(id.into()),
            federated_graph::Definition::Object(id) => Definition::Object(id.into()),
            federated_graph::Definition::Interface(id) => Definition::Interface(id.into()),
            federated_graph::Definition::Union(id) => Definition::Union(id.into()),
            federated_graph::Definition::Enum(id) => Definition::Enum(id.into()),
            federated_graph::Definition::InputObject(id) => Definition::InputObject(id.into()),
        }
    }
}

impl From<federated_graph::FieldType> for Type {
    fn from(field_type: federated_graph::FieldType) -> Self {
        Type {
            inner: field_type.kind.into(),
            wrapping: Wrapping {
                inner_is_required: field_type.inner_is_required,
                list_wrapping: field_type.list_wrappers.into_iter().rev().map(Into::into).collect(),
            },
        }
    }
}

impl From<federated_graph::ListWrapper> for ListWrapping {
    fn from(wrapper: federated_graph::ListWrapper) -> Self {
        match wrapper {
            federated_graph::ListWrapper::RequiredList => ListWrapping::RequiredList,
            federated_graph::ListWrapper::NullableList => ListWrapping::NullableList,
        }
    }
}

impl From<federated_graph::Interface> for Interface {
    fn from(interface: federated_graph::Interface) -> Self {
        Interface {
            name: interface.name.into(),
            description: None,
            interfaces: vec![],
            possible_types: vec![],
            composed_directives: interface.composed_directives.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<federated_graph::InterfaceField> for InterfaceField {
    fn from(interface_field: federated_graph::InterfaceField) -> Self {
        InterfaceField {
            interface_id: interface_field.interface_id.into(),
            field_id: interface_field.field_id.into(),
        }
    }
}

impl From<federated_graph::Enum> for Enum {
    fn from(value: federated_graph::Enum) -> Self {
        Enum {
            name: value.name.into(),
            description: None,
            values: value.values.into_iter().map(Into::into).collect(),
            composed_directives: value.composed_directives.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<federated_graph::Union> for Union {
    fn from(union: federated_graph::Union) -> Self {
        Union {
            name: union.name.into(),
            description: None,
            possible_types: union.members.into_iter().map(Into::into).collect(),
            composed_directives: union.composed_directives.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<federated_graph::Key> for Key {
    fn from(key: federated_graph::Key) -> Self {
        Key {
            subgraph_id: key.subgraph_id.into(),
            fields: key.fields.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<federated_graph::FieldSetItem> for FieldSetItem {
    fn from(selection: federated_graph::FieldSetItem) -> Self {
        FieldSetItem {
            field: selection.field.into(),
            selection_set: selection.subselection.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<federated_graph::EnumValue> for EnumValue {
    fn from(enum_value: federated_graph::EnumValue) -> Self {
        EnumValue {
            name: enum_value.value.into(),
            description: None,
            deprecation_reason: None,
            is_deprecated: false,
            composed_directives: enum_value.composed_directives.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<federated_graph::InputObjectField> for InputValue {
    fn from(field: federated_graph::InputObjectField) -> Self {
        InputValue {
            name: field.name.into(),
            description: None,
            type_id: field.field_type_id.into(),
            default_value: None,
        }
    }
}

macro_rules! from_id_newtypes {
    ($($from:ident => $name:ident,)*) => {
        $(
            impl From<federated_graph::$from> for $name {
                fn from(id: federated_graph::$from) -> Self {
                    $name::from(id.0)
                }
            }
        )*
    }
}

from_id_newtypes! {
    EnumId => EnumId,
    FieldId => FieldId,
    FieldTypeId => TypeId,
    InputObjectId => InputObjectId,
    InterfaceId => InterfaceId,
    ObjectId => ObjectId,
    ScalarId => ScalarId,
    StringId => StringId,
    SubgraphId => SubgraphId,
    UnionId => UnionId,
}

impl DataSourceId {
    fn from_subgraph_id(id: federated_graph::SubgraphId) -> Self {
        Self::from(id.0)
    }
}
