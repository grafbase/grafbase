use std::ops::Range;

pub use super::v3::{
    AuthorizedDirectiveId, Definition, DirectiveId, Directives, Enum, EnumId, EnumValue, EnumValueId, EnumValues,
    Field, FieldId, FieldProvides, FieldRequires, FieldSet, FieldSetItem, Fields, InputObject, InputObjectId,
    InputValueDefinitionId, InputValueDefinitionSet, InputValueDefinitionSetItem, InputValueDefinitions, Interface,
    InterfaceId, Key, Object, ObjectId, Override, OverrideLabel, OverrideSource, RootOperationTypes, Scalar, ScalarId,
    StringId, Subgraph, SubgraphId, Type, Union, UnionId, Wrapping, NO_DIRECTIVES, NO_ENUM_VALUE, NO_FIELDS,
    NO_INPUT_VALUE_DEFINITION,
};

#[derive(Clone)]
pub struct FederatedGraph {
    pub subgraphs: Vec<Subgraph>,
    pub root_operation_types: RootOperationTypes,
    pub objects: Vec<Object>,
    pub interfaces: Vec<Interface>,
    pub fields: Vec<Field>,

    pub enums: Vec<Enum>,
    pub unions: Vec<Union>,
    pub scalars: Vec<Scalar>,
    pub input_objects: Vec<InputObject>,
    pub enum_values: Vec<EnumValue>,

    /// All [input value definitions](http://spec.graphql.org/October2021/#InputValueDefinition) in the federated graph. Concretely, these are arguments of output fields, and input object fields.
    pub input_value_definitions: Vec<InputValueDefinition>,

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

impl std::fmt::Debug for FederatedGraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(std::any::type_name::<Self>()).finish_non_exhaustive()
    }
}

impl FederatedGraph {
    pub fn iter_interfaces(&self) -> impl ExactSizeIterator<Item = (InterfaceId, &Interface)> {
        self.interfaces
            .iter()
            .enumerate()
            .map(|(idx, interface)| (InterfaceId(idx), interface))
    }

    pub fn iter_objects(&self) -> impl ExactSizeIterator<Item = (ObjectId, &Object)> {
        self.objects
            .iter()
            .enumerate()
            .map(|(idx, object)| (ObjectId(idx), object))
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

#[derive(PartialEq, PartialOrd, Clone)]
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

#[derive(Default, serde::Serialize, serde::Deserialize, Clone, PartialEq, PartialOrd, Debug)]
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
    EnumValue(StringId),
    Object(Box<[(StringId, Value)]>),
    List(Box<[Value]>),
}

impl Value {
    pub fn is_list(&self) -> bool {
        matches!(self, Value::List(_))
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq, PartialOrd)]
pub struct AuthorizedDirective {
    pub fields: Option<FieldSet>,
    pub node: Option<FieldSet>,
    pub arguments: Option<InputValueDefinitionSet>,
    pub metadata: Option<Value>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq)]
pub struct InputValueDefinition {
    pub name: StringId,
    pub r#type: Type,
    pub directives: Directives,
    pub description: Option<StringId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<Value>,
}

impl Default for FederatedGraph {
    fn default() -> Self {
        FederatedGraph {
            subgraphs: Vec::new(),
            root_operation_types: RootOperationTypes {
                query: ObjectId(0),
                mutation: None,
                subscription: None,
            },
            objects: vec![Object {
                name: StringId(0),
                implements_interfaces: Vec::new(),
                keys: Vec::new(),
                composed_directives: NO_DIRECTIVES,
                fields: FieldId(0)..FieldId(2),
                description: None,
            }],
            interfaces: Vec::new(),
            fields: vec![
                Field {
                    name: StringId(1),
                    r#type: Type {
                        wrapping: Default::default(),
                        definition: Definition::Scalar(ScalarId(0)),
                    },
                    arguments: NO_INPUT_VALUE_DEFINITION,
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
                        definition: Definition::Scalar(ScalarId(0)),
                    },
                    arguments: NO_INPUT_VALUE_DEFINITION,
                    resolvable_in: Vec::new(),
                    provides: Vec::new(),
                    requires: Vec::new(),
                    overrides: Vec::new(),
                    composed_directives: NO_DIRECTIVES,
                    description: None,
                },
            ],
            enums: Vec::new(),
            unions: Vec::new(),
            scalars: Vec::new(),
            input_objects: Vec::new(),
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
    EnumId + enums + Enum,
    FieldId + fields + Field,
    InputValueDefinitionId + input_value_definitions + InputValueDefinition,
    InputObjectId + input_objects + InputObject,
    InterfaceId + interfaces + Interface,
    ObjectId + objects + Object,
    ScalarId + scalars + Scalar,
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
            input_objects,
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
        FederatedGraph {
            subgraphs,
            root_operation_types,
            objects,
            interfaces,
            fields,
            enums,
            unions,
            scalars,
            input_objects,
            enum_values,
            input_value_definitions: input_value_definitions
                .into_iter()
                .map(
                    |super::v3::InputValueDefinition {
                         name,
                         r#type,
                         directives,
                         description,
                         default,
                     }: super::v3::InputValueDefinition| InputValueDefinition {
                        name,
                        r#type,
                        directives,
                        description,
                        default: default.map(From::from),
                    },
                )
                .collect(),
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
            authorized_directives: authorized_directives
                .into_iter()
                .map(
                    |super::v3::AuthorizedDirective {
                         fields,
                         node,
                         arguments,
                         metadata,
                     }| AuthorizedDirective {
                        fields,
                        node,
                        arguments,
                        metadata: metadata.map(From::from),
                    },
                )
                .collect(),
            field_authorized_directives,
            object_authorized_directives,
            interface_authorized_directives,
        }
    }
}

impl From<super::v3::Value> for Value {
    fn from(value: super::v3::Value) -> Self {
        match value {
            super::v3::Value::Null => Value::Null,
            super::v3::Value::String(s) => Value::String(s),
            super::v3::Value::Int(i) => Value::Int(i),
            super::v3::Value::Float(i) => Value::Float(i),
            super::v3::Value::Boolean(b) => Value::Boolean(b),
            super::v3::Value::EnumValue(i) => Value::EnumValue(i),
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

impl std::ops::Index<InputValueDefinitions> for FederatedGraph {
    type Output = [InputValueDefinition];

    fn index(&self, index: InputValueDefinitions) -> &Self::Output {
        let (InputValueDefinitionId(start), len) = index;
        &self.input_value_definitions[start..(start + len)]
    }
}

impl std::ops::Index<EnumValues> for FederatedGraph {
    type Output = [EnumValue];

    fn index(&self, index: EnumValues) -> &Self::Output {
        let (EnumValueId(start), len) = index;
        &self.enum_values[start..(start + len)]
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
