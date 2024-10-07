mod debug;
mod enum_values;
mod field;
mod ids;
mod input_value_definitions;
mod iterators;
mod objects;
mod r#type;
mod type_definitions;
mod view;

use std::ops::Range;

pub use self::{
    enum_values::{EnumValue, EnumValueRecord},
    ids::*,
    input_value_definitions::*,
    r#type::{Definition, Type},
    type_definitions::{TypeDefinition, TypeDefinitionKind, TypeDefinitionRecord},
    view::{View, ViewNested},
};
pub use super::v3::{
    AuthorizedDirectiveId, DirectiveId, Directives, FieldId, Fields, InterfaceId, ObjectId, Override, OverrideLabel,
    OverrideSource, RootOperationTypes, StringId, Subgraph, SubgraphId, Union, UnionId, Wrapping, NO_DIRECTIVES,
    NO_FIELDS,
};

#[derive(Clone)]
pub struct FederatedGraph {
    pub subgraphs: Vec<Subgraph>,
    pub root_operation_types: RootOperationTypes,
    pub type_definitions: Vec<TypeDefinitionRecord>,
    pub objects: Vec<Object>,
    pub interfaces: Vec<Interface>,
    pub fields: Vec<Field>,

    pub unions: Vec<Union>,
    pub enum_values: Vec<EnumValueRecord>,

    /// All [input value definitions](http://spec.graphql.org/October2021/#InputValueDefinition) in the federated graph. Concretely, these are arguments of output fields, and input object fields.
    pub input_value_definitions: Vec<InputValueDefinitionRecord>,
    pub input_object_field_definitions: Vec<InputObjectFieldDefinitionRecord>,
    pub argument_definitions: Vec<ArgumentDefinitionRecord>,

    /// All the strings in the federated graph, deduplicated.
    pub strings: Vec<String>,

    /// All composed directive instances (not definitions) in a federated graph.
    pub directives: Vec<Directive>,

    /// All @authorized directives
    pub authorized_directives: Vec<AuthorizedDirective>,
    pub field_authorized_directives: Vec<(FieldId, AuthorizedDirectiveId)>,
    pub object_authorized_directives: Vec<(ObjectId, AuthorizedDirectiveId)>,
    pub interface_authorized_directives: Vec<(InterfaceId, AuthorizedDirectiveId)>,
}

impl FederatedGraph {
    pub fn definition_name(&self, definition: Definition) -> &str {
        let name_id = match definition {
            Definition::Scalar(scalar_id) => self[scalar_id].name,
            Definition::Object(object_id) => self.at(object_id).then(|obj| obj.type_definition_id).name,
            Definition::Interface(interface_id) => {
                self.at(interface_id)
                    .then(|interface| interface.type_definition_id)
                    .name
            }
            Definition::Union(union_id) => self[union_id].name,
            Definition::Enum(enum_id) => self[enum_id].name,
            Definition::InputObject(input_object_id) => self[input_object_id].name,
        };

        &self[name_id]
    }

    pub fn iter_interfaces(&self) -> impl ExactSizeIterator<Item = View<InterfaceId, &Interface>> {
        (0..self.interfaces.len()).map(|idx| self.view(InterfaceId::from(idx)))
    }

    pub fn iter_objects(&self) -> impl ExactSizeIterator<Item = View<ObjectId, &Object>> {
        (0..self.objects.len()).map(|idx| self.view(ObjectId::from(idx)))
    }

    pub fn iter_type_definitions(&self) -> impl Iterator<Item = TypeDefinition<'_>> {
        self.type_definitions
            .iter()
            .enumerate()
            .map(|(idx, _)| self.at(TypeDefinitionId::from(idx)))
    }

    pub fn iter_enums(&self) -> impl Iterator<Item = TypeDefinition<'_>> {
        self.iter_type_definitions().filter(|record| record.kind.is_enum())
    }

    pub fn iter_scalars(&self) -> impl Iterator<Item = TypeDefinition<'_>> {
        self.iter_type_definitions().filter(|record| record.kind.is_scalar())
    }

    pub fn object_authorized_directives(&self, object_id: ObjectId) -> impl Iterator<Item = &AuthorizedDirective> {
        let start = self
            .object_authorized_directives
            .partition_point(|(needle, _)| *needle < object_id);

        self.object_authorized_directives[start..]
            .iter()
            .take_while(move |(needle, _)| *needle == object_id)
            .map(move |(_, authorized_directive_id)| &self[*authorized_directive_id])
    }

    pub fn interface_authorized_directives(
        &self,
        interface_id: InterfaceId,
    ) -> impl Iterator<Item = &AuthorizedDirective> {
        let start = self
            .interface_authorized_directives
            .partition_point(|(needle, _)| *needle < interface_id);

        self.interface_authorized_directives[start..]
            .iter()
            .take_while(move |(needle, _)| *needle == interface_id)
            .map(move |(_, authorized_directive_id)| &self[*authorized_directive_id])
    }
}

#[derive(PartialEq, PartialOrd, Clone, Debug)]
pub enum Directive {
    Authenticated,
    Deprecated {
        reason: Option<StringId>,
    },
    Inaccessible,
    Policy(Vec<Vec<StringId>>),
    RequiresScopes(Vec<Vec<StringId>>),

    Other {
        name: StringId,
        arguments: Vec<(StringId, Value)>,
    },
}

#[derive(Default, Clone, PartialEq, PartialOrd, Debug)]
#[allow(clippy::enum_variant_names)]
pub enum Value {
    #[default]
    Null,
    String(StringId),
    Int(i64),
    Float(f64),
    Boolean(bool),
    /// Different from `String`.
    ///
    /// `@tag(name: "SOMETHING")` vs `@tag(name: SOMETHING)`
    ///
    /// FIXME: This is currently required because we do not keep accurate track of the directives in use in the schema, but we should strive towards removing UnboundEnumValue in favour of EnumValue.
    UnboundEnumValue(StringId),
    EnumValue(EnumValueId),
    Object(Box<[(StringId, Value)]>),
    List(Box<[Value]>),
}

#[derive(Clone)]
pub struct Object {
    pub type_definition_id: TypeDefinitionId,

    pub implements_interfaces: Vec<InterfaceId>,

    pub join_implements: Vec<(SubgraphId, InterfaceId)>,

    pub keys: Vec<Key>,
    pub fields: Fields,
}

#[derive(Clone)]
pub struct Interface {
    pub type_definition_id: TypeDefinitionId,
    pub implements_interfaces: Vec<InterfaceId>,

    /// All keys, for entity interfaces.
    pub keys: Vec<Key>,
    pub fields: Fields,
    pub join_implements: Vec<(SubgraphId, InterfaceId)>,
}

#[derive(Clone)]
pub struct Field {
    pub name: StringId,
    pub r#type: Type,

    /// This is populated only of fields of entities. The Vec includes all subgraphs the field can
    /// be resolved in. For a regular field of an entity, it will be one subgraph, the subgraph
    /// where the entity field is defined. For a shareable field in an entity, this contains the
    /// subgraphs where the shareable field is defined on the entity. It may not be all the
    /// subgraphs.
    ///
    /// On fields of value types and input types, this is empty.
    pub resolvable_in: Vec<SubgraphId>,

    /// See [FieldProvides].
    pub provides: Vec<FieldProvides>,

    /// See [FieldRequires]
    pub requires: Vec<FieldRequires>,

    /// See [Override].
    pub overrides: Vec<Override>,

    /// All directives that made it through composition.
    pub composed_directives: Directives,

    pub description: Option<StringId>,
}

impl Value {
    pub fn is_list(&self) -> bool {
        matches!(self, Value::List(_))
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }
}

#[derive(Clone, PartialEq, PartialOrd)]
pub struct AuthorizedDirective {
    pub fields: Option<SelectionSet>,
    pub node: Option<SelectionSet>,
    pub arguments: Option<InputValueDefinitionSet>,
    pub metadata: Option<Value>,
}

/// Represents an `@provides` directive on a field in a subgraph.
#[derive(Clone)]
pub struct FieldProvides {
    pub subgraph_id: SubgraphId,
    pub fields: SelectionSet,
}

/// Represents an `@requires` directive on a field in a subgraph.
#[derive(Clone)]
pub struct FieldRequires {
    pub subgraph_id: SubgraphId,
    pub fields: SelectionSet,
}

pub type SelectionSet = Vec<Selection>;

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum Selection {
    Field {
        field: FieldId,
        arguments: Vec<(InputValueDefinitionId, Value)>,
        subselection: SelectionSet,
    },
    InlineFragment {
        on: Definition,
        subselection: SelectionSet,
    },
}

#[derive(Clone, Debug)]
pub struct Key {
    /// The subgraph that can resolve the entity with the fields in [Key::fields].
    pub subgraph_id: SubgraphId,

    /// Corresponds to the fields argument in an `@key` directive.
    pub fields: SelectionSet,

    /// Correspond to the `@join__type(isInterfaceObject: true)` directive argument.
    pub is_interface_object: bool,

    pub resolvable: bool,
}

impl Default for FederatedGraph {
    fn default() -> Self {
        FederatedGraph {
            subgraphs: Vec::new(),
            type_definitions: vec![TypeDefinitionRecord {
                name: StringId::from(0),
                description: None,
                directives: NO_DIRECTIVES,
                kind: TypeDefinitionKind::Object,
            }],
            root_operation_types: RootOperationTypes {
                query: ObjectId(0),
                mutation: None,
                subscription: None,
            },
            objects: vec![Object {
                type_definition_id: TypeDefinitionId::from(0),
                implements_interfaces: Vec::new(),
                join_implements: Vec::new(),
                keys: Vec::new(),
                fields: FieldId(0)..FieldId(2),
            }],
            interfaces: Vec::new(),
            fields: vec![
                Field {
                    name: StringId(1),
                    r#type: Type {
                        wrapping: Default::default(),
                        definition: Definition::Scalar(0usize.into()),
                    },
                    resolvable_in: Vec::new(),
                    provides: Vec::new(),
                    requires: Vec::new(),
                    overrides: Vec::new(),
                    composed_directives: NO_DIRECTIVES,
                    description: None,
                },
                Field {
                    name: StringId(2),
                    r#type: Type {
                        wrapping: Default::default(),
                        definition: Definition::Scalar(0usize.into()),
                    },
                    resolvable_in: Vec::new(),
                    provides: Vec::new(),
                    requires: Vec::new(),
                    overrides: Vec::new(),
                    composed_directives: NO_DIRECTIVES,
                    description: None,
                },
            ],
            unions: Vec::new(),
            input_object_field_definitions: Vec::new(),
            argument_definitions: Vec::new(),
            enum_values: Vec::new(),
            input_value_definitions: Vec::new(),
            strings: ["Query", "__type", "__schema"]
                .into_iter()
                .map(|string| string.to_owned())
                .collect(),
            directives: Vec::new(),
            authorized_directives: Vec::new(),
            field_authorized_directives: Vec::new(),
            object_authorized_directives: Vec::new(),
            interface_authorized_directives: Vec::new(),
        }
    }
}

macro_rules! id_newtypes {
    ($($name:ident + $storage:ident + $out:ident,)*) => {
        $(
            impl std::ops::Index<$name> for FederatedGraph {
                type Output = $out;

                fn index(&self, index: $name) -> &$out {
                    &self.$storage[index.0]
                }
            }

            impl std::ops::IndexMut<$name> for FederatedGraph {
                fn index_mut(&mut self, index: $name) -> &mut $out {
                    &mut self.$storage[index.0]
                }
            }
        )*
    }
}

id_newtypes! {
    AuthorizedDirectiveId + authorized_directives + AuthorizedDirective,
    FieldId + fields + Field,
    InterfaceId + interfaces + Interface,
    ObjectId + objects + Object,
    StringId + strings + String,
    SubgraphId + subgraphs + Subgraph,
    UnionId + unions + Union,
}

impl From<super::FederatedGraphV3> for FederatedGraph {
    fn from(
        crate::FederatedGraphV3 {
            subgraphs,
            root_operation_types,
            objects,
            interfaces,
            fields,
            enums,
            unions,
            scalars,
            input_objects: _,
            enum_values,
            input_value_definitions,
            strings,
            directives,
            authorized_directives,
            field_authorized_directives,
            object_authorized_directives,
            interface_authorized_directives,
        }: super::FederatedGraphV3,
    ) -> Self {
        use std::collections::HashMap;

        let mut type_definitions = Vec::new(); // could make better, but I don't think this is ever going to get called
        let mut definitions_map: HashMap<super::v3::Definition, Definition> = HashMap::new();

        for (idx, object) in objects.iter().enumerate() {
            let id = ObjectId::from(idx);
            type_definitions.push(TypeDefinitionRecord {
                name: object.name,
                description: object.description,
                directives: object.composed_directives,
                kind: TypeDefinitionKind::Object,
            });
            definitions_map.insert(super::v3::Definition::Object(id), Definition::Object(id));
        }

        for (idx, interface) in interfaces.iter().enumerate() {
            let id = InterfaceId::from(idx);
            type_definitions.push(TypeDefinitionRecord {
                name: interface.name,
                description: interface.description,
                directives: interface.composed_directives,
                kind: TypeDefinitionKind::Interface,
            });
            definitions_map.insert(super::v3::Definition::Interface(id), Definition::Interface(id));
        }

        for (idx, scalar) in scalars.iter().enumerate() {
            let id = TypeDefinitionId::from(type_definitions.len());
            type_definitions.push(TypeDefinitionRecord {
                name: scalar.name,
                description: scalar.description,
                directives: scalar.composed_directives,
                kind: TypeDefinitionKind::Scalar,
            });
            definitions_map.insert(super::v3::Definition::Scalar(idx.into()), Definition::Scalar(id));
        }

        let mut new_enum_values = Vec::new();
        for r#enum in enums {
            let enum_id = TypeDefinitionId::from(type_definitions.len());
            type_definitions.push(TypeDefinitionRecord {
                name: r#enum.name,
                description: r#enum.description,
                directives: r#enum.composed_directives,
                kind: TypeDefinitionKind::Enum,
            });

            for enum_value in &enum_values[r#enum.values.0 .0..(r#enum.values.0 .0 + r#enum.values.1)] {
                new_enum_values.push(EnumValueRecord {
                    enum_id,
                    value: enum_value.value,
                    composed_directives: enum_value.composed_directives,
                    description: enum_value.description,
                })
            }
        }

        let mut type_definitions_counter = 0;

        FederatedGraph {
            type_definitions,
            subgraphs,
            root_operation_types,
            objects: objects
                .into_iter()
                .map(
                    |super::v3::Object {
                         implements_interfaces,
                         keys,
                         fields,
                         ..
                     }| {
                        let object = Object {
                            type_definition_id: type_definitions_counter.into(),
                            implements_interfaces,
                            join_implements: Vec::new(),
                            keys: convert_keys(keys),
                            fields,
                        };
                        type_definitions_counter += 1;
                        object
                    },
                )
                .collect(),
            interfaces: interfaces
                .into_iter()
                .map(
                    |super::v3::Interface {
                         implements_interfaces,
                         keys,
                         fields,
                         ..
                     }| {
                        let interface = Interface {
                            type_definition_id: type_definitions_counter.into(),
                            implements_interfaces,
                            keys: convert_keys(keys),
                            fields,
                            join_implements: Vec::new(),
                        };
                        type_definitions_counter += 1;
                        interface
                    },
                )
                .collect(),
            fields: fields
                .into_iter()
                .map(
                    |super::v3::Field {
                         name,
                         r#type,
                         arguments,
                         resolvable_in,
                         provides,
                         requires,
                         overrides,
                         composed_directives,
                         description,
                     }| Field {
                        name,
                        r#type: Type {
                            definition: definitions_map
                                .get(&r#type.definition)
                                .copied()
                                .unwrap_or_else(|| Definition::Scalar(TypeDefinitionId::from(0))),
                            wrapping: r#type.wrapping,
                        },
                        resolvable_in,
                        provides: provides
                            .into_iter()
                            .map(|super::v1::FieldProvides { subgraph_id, fields }| FieldProvides {
                                subgraph_id,
                                fields: field_set_to_selection_set(fields),
                            })
                            .collect(),
                        requires: requires
                            .into_iter()
                            .map(|super::v1::FieldRequires { subgraph_id, fields }| FieldRequires {
                                subgraph_id,
                                fields: field_set_to_selection_set(fields),
                            })
                            .collect(),
                        overrides,
                        composed_directives,
                        description,
                    },
                )
                .collect(),
            unions,
            enum_values: new_enum_values,
            input_value_definitions: Vec::new(),
            strings,
            directives: directives
                .into_iter()
                .map(|directive| match directive {
                    super::v3::Directive::Authenticated => Directive::Authenticated,
                    super::v3::Directive::Deprecated { reason } => Directive::Deprecated { reason },
                    super::v3::Directive::Inaccessible => Directive::Inaccessible,
                    super::v3::Directive::Policy(policy) => Directive::Policy(policy),
                    super::v3::Directive::RequiresScopes(scopes) => Directive::RequiresScopes(scopes),
                    super::v3::Directive::Other { name, arguments } => Directive::Other {
                        name,
                        arguments: arguments.into_iter().map(|(key, value)| (key, value.into())).collect(),
                    },
                })
                .collect(),
            authorized_directives: Vec::new(),
            field_authorized_directives,
            object_authorized_directives,
            interface_authorized_directives,
            input_object_field_definitions: Vec::new(),
            argument_definitions: Vec::new(),
        }
    }
}

fn convert_keys(keys: Vec<super::v1::Key>) -> Vec<Key> {
    keys.into_iter()
        .map(
            |super::v1::Key {
                 subgraph_id,
                 fields,
                 is_interface_object,
                 resolvable,
             }| Key {
                subgraph_id,
                fields: field_set_to_selection_set(fields),
                is_interface_object,
                resolvable,
            },
        )
        .collect()
}

fn field_set_to_selection_set(field_set: Vec<super::v1::FieldSetItem>) -> SelectionSet {
    field_set
        .into_iter()
        .map(
            |super::v1::FieldSetItem {
                 field,
                 arguments,
                 subselection,
             }| {
                Selection::Field {
                    field,
                    arguments: arguments
                        .into_iter()
                        .map(|(k, v)| (k, super::v3::Value::from((v, &[] as &[String])).into()))
                        .collect(),
                    subselection: field_set_to_selection_set(subselection),
                }
            },
        )
        .collect()
}

impl From<super::v3::Value> for Value {
    fn from(value: super::v3::Value) -> Self {
        match value {
            super::v3::Value::Null => Value::Null,
            super::v3::Value::String(s) => Value::String(s),
            super::v3::Value::Int(i) => Value::Int(i),
            super::v3::Value::Float(i) => Value::Float(i),
            super::v3::Value::Boolean(b) => Value::Boolean(b),
            super::v3::Value::EnumValue(i) => Value::String(i),
            super::v3::Value::Object(obj) => Value::Object(
                obj.iter()
                    .map(|(k, v)| (*k, v.clone().into()))
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            ),
            super::v3::Value::List(list) => Value::List(
                list.iter()
                    .map(|inner| inner.clone().into())
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            ),
        }
    }
}

impl std::ops::Index<Directives> for FederatedGraph {
    type Output = [Directive];

    fn index(&self, index: Directives) -> &Self::Output {
        let (DirectiveId(start), len) = index;
        &self.directives[start..(start + len)]
    }
}

impl std::ops::Index<Fields> for FederatedGraph {
    type Output = [Field];

    fn index(&self, index: Fields) -> &Self::Output {
        let Range {
            start: FieldId(start),
            end: FieldId(end),
        } = index;
        &self.fields[start..end]
    }
}
