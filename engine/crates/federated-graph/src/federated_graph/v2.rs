pub use super::v1::{
    Definition, EnumId, FieldId, FieldProvides, FieldRequires, FieldSet, FieldSetItem, FieldType, InputObjectId,
    InterfaceField, InterfaceId, Key, ListWrapper, ObjectField, ObjectId, Override, OverrideLabel, OverrideSource,
    RootOperationTypes, ScalarId, StringId, Subgraph, SubgraphId, TypeId, UnionId,
};

/// A composed federated graph.
///
/// ## API contract
///
/// Guarantees:
///
/// - All the identifiers are correct.
///
/// Does not guarantee:
///
/// - The ordering of items inside each `Vec`.
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct FederatedGraphV2 {
    pub subgraphs: Vec<Subgraph>,

    pub root_operation_types: RootOperationTypes,
    pub objects: Vec<Object>,
    pub object_fields: Vec<ObjectField>,

    pub interfaces: Vec<Interface>,
    pub interface_fields: Vec<InterfaceField>,

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

    /// All the field types in the federated graph, deduplicated.
    pub field_types: Vec<FieldType>,

    /// All composed directive instances (not definitions) in a federated graph.
    pub directives: Vec<Directive>,
}

impl std::fmt::Debug for FederatedGraphV2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(std::any::type_name::<Self>()).finish_non_exhaustive()
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct InputValueDefinition {
    pub name: StringId,
    pub type_id: TypeId,
    pub directives: Directives,
    pub description: Option<StringId>,
}

#[derive(Default, serde::Serialize, serde::Deserialize, Clone, PartialEq, PartialOrd, Debug)]
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
    pub fn as_list(&self) -> Option<&[Value]> {
        if let Self::List(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_string(&self) -> Option<&StringId> {
        if let Self::String(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn is_list(&self) -> bool {
        matches!(self, Self::List(_))
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq, PartialOrd)]
pub enum Directive {
    Inaccessible,
    Deprecated {
        reason: Option<StringId>,
    },
    Other {
        name: StringId,
        arguments: Vec<(StringId, Value)>,
    },
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Enum {
    pub name: StringId,
    pub values: EnumValues,

    /// All directives that made it through composition. Notably includes `@tag`.
    pub composed_directives: Directives,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<StringId>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct EnumValue {
    pub value: StringId,

    /// All directives that made it through composition. Notably includes `@tag`.
    pub composed_directives: Directives,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<StringId>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Field {
    pub name: StringId,
    pub field_type_id: TypeId,

    pub arguments: InputValueDefinitions,

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

    /// All directives that made it through composition. Notably includes `@tag`.
    pub composed_directives: Directives,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<StringId>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Object {
    pub name: StringId,

    pub implements_interfaces: Vec<InterfaceId>,

    #[serde(rename = "resolvable_keys")]
    pub keys: Vec<Key>,

    /// All directives that made it through composition. Notably includes `@tag`.
    pub composed_directives: Directives,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<StringId>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Interface {
    pub name: StringId,

    pub implements_interfaces: Vec<InterfaceId>,

    /// All keys, for entity interfaces.
    #[serde(rename = "resolvable_keys")]
    pub keys: Vec<Key>,

    /// All directives that made it through composition. Notably includes `@tag`.
    pub composed_directives: Directives,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<StringId>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Scalar {
    pub name: StringId,

    /// All directives that made it through composition. Notably includes `@tag`.
    pub composed_directives: Directives,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<StringId>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Union {
    pub name: StringId,
    pub members: Vec<ObjectId>,

    /// All directives that made it through composition. Notably includes `@tag`.
    pub composed_directives: Directives,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<StringId>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct InputObject {
    pub name: StringId,

    pub fields: InputValueDefinitions,

    /// All directives that made it through composition. Notably includes `@tag`.
    pub composed_directives: Directives,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<StringId>,
}

/// A (start, len) range in FederatedSchema.
pub type Directives = (DirectiveId, usize);

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct DirectiveId(pub usize);

impl From<DirectiveId> for usize {
    fn from(DirectiveId(index): DirectiveId) -> Self {
        index
    }
}
pub const NO_DIRECTIVES: Directives = (DirectiveId(0), 0);

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct InputValueDefinitionId(pub usize);

/// A (start, len) range in FederatedSchema.
pub type InputValueDefinitions = (InputValueDefinitionId, usize);

impl From<InputValueDefinitionId> for usize {
    fn from(InputValueDefinitionId(index): InputValueDefinitionId) -> Self {
        index
    }
}

impl From<usize> for InputValueDefinitionId {
    fn from(index: usize) -> Self {
        InputValueDefinitionId(index)
    }
}

pub const NO_INPUT_VALUE_DEFINITION: InputValueDefinitions = (InputValueDefinitionId(0), 0);

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct EnumValueId(pub usize);

impl From<EnumValueId> for usize {
    fn from(EnumValueId(index): EnumValueId) -> Self {
        index
    }
}

impl From<usize> for EnumValueId {
    fn from(index: usize) -> Self {
        EnumValueId(index)
    }
}

/// A (start, len) range in FederatedSchema.
pub type EnumValues = (EnumValueId, usize);

pub const NO_ENUM_VALUE: EnumValues = (EnumValueId(0), 0);

impl std::ops::Index<Directives> for FederatedGraphV2 {
    type Output = [Directive];

    fn index(&self, index: Directives) -> &Self::Output {
        let (DirectiveId(start), len) = index;
        &self.directives[start..(start + len)]
    }
}

impl std::ops::Index<InputValueDefinitions> for FederatedGraphV2 {
    type Output = [InputValueDefinition];

    fn index(&self, index: InputValueDefinitions) -> &Self::Output {
        let (InputValueDefinitionId(start), len) = index;
        &self.input_value_definitions[start..(start + len)]
    }
}

impl std::ops::Index<EnumValues> for FederatedGraphV2 {
    type Output = [EnumValue];

    fn index(&self, index: EnumValues) -> &Self::Output {
        let (EnumValueId(start), len) = index;
        &self.enum_values[start..(start + len)]
    }
}

macro_rules! id_newtypes {
    ($($name:ident + $storage:ident + $out:ident,)*) => {
        $(
            impl std::ops::Index<$name> for FederatedGraphV2 {
                type Output = $out;

                fn index(&self, index: $name) -> &$out {
                    &self.$storage[index.0]
                }
            }

            impl std::ops::IndexMut<$name> for FederatedGraphV2 {
                fn index_mut(&mut self, index: $name) -> &mut $out {
                    &mut self.$storage[index.0]
                }
            }
        )*
    }
}

id_newtypes! {
    EnumId + enums + Enum,
    FieldId + fields + Field,
    TypeId + field_types + FieldType,
    InputObjectId + input_objects + InputObject,
    InterfaceId + interfaces + Interface,
    ObjectId + objects + Object,
    ScalarId + scalars + Scalar,
    StringId + strings + String,
    SubgraphId + subgraphs + Subgraph,
    UnionId + unions + Union,
}

impl From<super::v1::FederatedGraphV1> for FederatedGraphV2 {
    fn from(
        super::v1::FederatedGraphV1 {
            subgraphs,
            root_operation_types,
            objects,
            object_fields,
            interfaces,
            interface_fields,
            fields,
            enums,
            unions,
            scalars,
            input_objects,
            strings,
            field_types,
        }: super::v1::FederatedGraphV1,
    ) -> Self {
        let mut directives = vec![];
        let mut input_value_definitions = vec![];
        let mut enum_values = vec![];

        let convert_directives = |original: Vec<super::v1::Directive>, directives: &mut Vec<Directive>| -> Directives {
            let start = directives.len();

            for directive in original {
                let directive = if strings[directive.name.0] == "inaccessible" {
                    Directive::Inaccessible
                } else {
                    Directive::Other {
                        name: directive.name,
                        arguments: directive
                            .arguments
                            .into_iter()
                            .map(|(name, value)| (name, (value, strings.as_slice()).into()))
                            .collect(),
                    }
                };

                directives.push(directive);
            }

            let end = directives.len();
            (DirectiveId(start), end - start)
        };

        let convert_arguments = |original: Vec<super::v1::FieldArgument>,
                                 input_value_definitions: &mut Vec<InputValueDefinition>,
                                 directives: &mut Vec<Directive>|
         -> InputValueDefinitions {
            let start = input_value_definitions.len();

            for super::v1::FieldArgument {
                name,
                type_id,
                composed_directives,
                description,
            } in original
            {
                input_value_definitions.push(InputValueDefinition {
                    name,
                    type_id,
                    directives: convert_directives(composed_directives, directives),
                    description,
                });
            }

            (InputValueDefinitionId(start), input_value_definitions.len() - start)
        };

        let convert_input_object_fields = |original: Vec<super::v1::InputObjectField>,
                                           input_value_definitions: &mut Vec<InputValueDefinition>,
                                           directives: &mut Vec<Directive>|
         -> InputValueDefinitions {
            let start = input_value_definitions.len();

            for super::v1::InputObjectField {
                name,
                field_type_id,
                composed_directives,
                description,
            } in original
            {
                input_value_definitions.push(InputValueDefinition {
                    name,
                    type_id: field_type_id,
                    directives: convert_directives(composed_directives, directives),
                    description,
                });
            }

            (InputValueDefinitionId(start), input_value_definitions.len() - start)
        };

        let convert_enum_values = |original: Vec<super::v1::EnumValue>,
                                   enum_values: &mut Vec<EnumValue>,
                                   directives: &mut Vec<Directive>|
         -> EnumValues {
            let start = enum_values.len();

            for super::v1::EnumValue {
                value,
                composed_directives,
                description,
            } in original
            {
                enum_values.push(EnumValue {
                    value,
                    composed_directives: convert_directives(composed_directives, directives),
                    description,
                })
            }

            (EnumValueId(start), enum_values.len() - start)
        };

        FederatedGraphV2 {
            subgraphs,
            root_operation_types,
            objects: objects
                .into_iter()
                .map(
                    |super::v1::Object {
                         name,
                         implements_interfaces,
                         keys,
                         composed_directives,
                         description,
                     }| Object {
                        name,
                        implements_interfaces,
                        keys,
                        composed_directives: convert_directives(composed_directives, &mut directives),
                        description,
                    },
                )
                .collect(),
            object_fields,
            interfaces: interfaces
                .into_iter()
                .map(
                    |super::v1::Interface {
                         name,
                         implements_interfaces,
                         keys,
                         composed_directives,
                         description,
                     }| Interface {
                        name,
                        implements_interfaces,
                        keys,
                        composed_directives: convert_directives(composed_directives, &mut directives),
                        description,
                    },
                )
                .collect(),
            interface_fields,
            fields: fields
                .into_iter()
                .map(
                    |super::v1::Field {
                         name,
                         field_type_id,
                         resolvable_in,
                         provides,
                         requires,
                         overrides,
                         arguments,
                         composed_directives,
                         description,
                     }| Field {
                        name,
                        field_type_id,
                        arguments: convert_arguments(arguments, &mut input_value_definitions, &mut directives),
                        resolvable_in,
                        provides,
                        requires,
                        overrides,
                        composed_directives: convert_directives(composed_directives, &mut directives),
                        description,
                    },
                )
                .collect(),
            enums: enums
                .into_iter()
                .map(
                    |super::v1::Enum {
                         name,
                         values,
                         composed_directives,
                         description,
                     }| Enum {
                        name,
                        values: convert_enum_values(values, &mut enum_values, &mut directives),
                        composed_directives: convert_directives(composed_directives, &mut directives),
                        description,
                    },
                )
                .collect(),
            unions: unions
                .into_iter()
                .map(
                    |super::v1::Union {
                         name,
                         members,
                         composed_directives,
                         description,
                     }| Union {
                        name,
                        members,
                        composed_directives: convert_directives(composed_directives, &mut directives),
                        description,
                    },
                )
                .collect(),
            scalars: scalars
                .into_iter()
                .map(
                    |super::v1::Scalar {
                         name,
                         composed_directives,
                         description,
                     }| Scalar {
                        name,
                        composed_directives: convert_directives(composed_directives, &mut directives),
                        description,
                    },
                )
                .collect(),
            input_objects: input_objects
                .into_iter()
                .map(
                    |super::v1::InputObject {
                         name,
                         fields,
                         composed_directives,
                         description,
                     }| InputObject {
                        name,
                        fields: convert_input_object_fields(fields, &mut input_value_definitions, &mut directives),
                        composed_directives: convert_directives(composed_directives, &mut directives),
                        description,
                    },
                )
                .collect(),
            input_value_definitions,
            strings,
            field_types,
            directives,
            enum_values,
        }
    }
}

impl From<(super::v1::Value, &[String])> for Value {
    fn from((value, strings): (super::v1::Value, &[String])) -> Self {
        match value {
            super::v1::Value::String(v) => Value::String(v),
            super::v1::Value::Int(v) => Value::Int(v),
            super::v1::Value::Float(v) => Value::Float(strings[v.0].parse().unwrap()),
            super::v1::Value::Boolean(v) => Value::Boolean(v),
            super::v1::Value::EnumValue(v) => Value::EnumValue(v),
            super::v1::Value::Object(v) => {
                Value::Object(v.into_iter().map(|(k, v)| (k, (v, strings).into())).collect())
            }
            super::v1::Value::List(v) => Value::List(v.into_iter().map(|v| (v, strings).into()).collect()),
        }
    }
}
