use std::collections::HashMap;

// All of that should be in federated_graph actually.
use super::*;

impl From<federated_graph::FederatedGraph> for Schema {
    fn from(graph: federated_graph::FederatedGraph) -> Self {
        let mut out = Schema {
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
            field_types: graph.field_types.into_iter().map(Into::into).collect(),
            interfaces: graph.interfaces.into_iter().map(Into::into).collect(),
            interface_fields: graph.interface_fields.into_iter().map(Into::into).collect(),
            enums: graph.enums.into_iter().map(Into::into).collect(),
            unions: graph.unions.into_iter().map(Into::into).collect(),
            scalars: graph.scalars.into_iter().map(Into::into).collect(),
            input_objects: graph.input_objects.into_iter().map(Into::into).collect(),
            strings: graph.strings,
            resolvers: vec![],
        };
        out.object_fields.sort_unstable();
        out.interface_fields.sort_unstable_by_key(|field| field.interface_id);

        // Yeah it's ugly, conversion should be cleaned up once we got it working I guess.
        let mut resolvers = HashMap::<Resolver, ResolverId>::new();
        for field in graph.fields {
            let mut field_resolvers = vec![];
            let mut field_requires = field
                .requires
                .into_iter()
                .map(|federated_graph::FieldRequires { subgraph_id, fields }| (subgraph_id, fields))
                .collect::<HashMap<_, _>>();
            if let Some(subgraph_id) = field.resolvable_in {
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
            out.fields.push(Field {
                name: field.name.into(),
                field_type_id: field.field_type_id.into(),
                resolvers: field_resolvers,
                provides: field.provides.into_iter().map(Into::into).collect(),
                arguments: field.arguments.into_iter().map(Into::into).collect(),
                composed_directives: field.composed_directives.into_iter().map(Into::into).collect(),
            });
        }
        let mut resolvers = resolvers.into_iter().collect::<Vec<_>>();
        resolvers.sort_unstable_by_key(|(_, resolver_id)| *resolver_id);
        out.resolvers = resolvers.into_iter().map(|(resolver, _)| resolver).collect();
        out
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
            implements_interfaces: object.implements_interfaces.into_iter().map(Into::into).collect(),
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

impl From<federated_graph::FieldArgument> for FieldArgument {
    fn from(argument: federated_graph::FieldArgument) -> Self {
        FieldArgument {
            name: argument.name.into(),
            type_id: argument.type_id.into(),
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

impl From<federated_graph::FieldType> for FieldType {
    fn from(field_type: federated_graph::FieldType) -> Self {
        FieldType {
            kind: field_type.kind.into(),
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
            values: value.values.into_iter().map(Into::into).collect(),
            composed_directives: value.composed_directives.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<federated_graph::Union> for Union {
    fn from(union: federated_graph::Union) -> Self {
        Union {
            name: union.name.into(),
            members: union.members.into_iter().map(Into::into).collect(),
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
            subselection: selection.subselection.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<federated_graph::EnumValue> for EnumValue {
    fn from(enum_value: federated_graph::EnumValue) -> Self {
        EnumValue {
            value: enum_value.value.into(),
            composed_directives: enum_value.composed_directives.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<federated_graph::Scalar> for Scalar {
    fn from(scalar: federated_graph::Scalar) -> Self {
        Scalar {
            name: scalar.name.into(),
            composed_directives: scalar.composed_directives.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<federated_graph::InputObject> for InputObject {
    fn from(input_object: federated_graph::InputObject) -> Self {
        InputObject {
            name: input_object.name.into(),
            fields: input_object.fields.into_iter().map(Into::into).collect(),
            composed_directives: input_object.composed_directives.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<federated_graph::InputObjectField> for InputObjectField {
    fn from(field: federated_graph::InputObjectField) -> Self {
        InputObjectField {
            name: field.name.into(),
            field_type_id: field.field_type_id.into(),
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
    FieldTypeId => FieldTypeId,
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
