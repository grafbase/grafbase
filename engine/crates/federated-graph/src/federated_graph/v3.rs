use std::ops::Range;

pub use super::v2::{
    Definition, DirectiveId, Directives, Enum, EnumId, EnumValue, EnumValueId, EnumValues, FieldId, FieldProvides,
    FieldRequires, FieldSet, FieldSetItem, InputObject, InputObjectId, InputValueDefinitionId, InputValueDefinitions,
    InterfaceId, Key, ObjectId, Override, OverrideSource, RootOperationTypes, Scalar, ScalarId, StringId, Subgraph,
    SubgraphId, Union, UnionId, Value, NO_DIRECTIVES, NO_ENUM_VALUE, NO_INPUT_VALUE_DEFINITION,
};
pub use wrapping::Wrapping;

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
pub struct FederatedGraphV3 {
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
}

#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq, PartialOrd)]
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

#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq)]
pub struct Type {
    pub wrapping: Wrapping,
    pub definition: Definition,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Field {
    pub name: StringId,
    pub r#type: Type,

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

    /// All directives that made it through composition.
    pub composed_directives: Directives,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<StringId>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Object {
    pub name: StringId,

    pub implements_interfaces: Vec<InterfaceId>,

    pub keys: Vec<Key>,

    /// All directives that made it through composition.
    pub composed_directives: Directives,

    pub fields: Fields,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<StringId>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Interface {
    pub name: StringId,

    pub implements_interfaces: Vec<InterfaceId>,

    /// All keys, for entity interfaces.
    pub keys: Vec<Key>,

    /// All directives that made it through composition.
    pub composed_directives: Directives,

    pub fields: Fields,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<StringId>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq)]
pub struct InputValueDefinition {
    pub name: StringId,
    pub r#type: Type,
    pub directives: Directives,
    pub description: Option<StringId>,
}

/// A (start, end) range in FederatedGraph::fields.
pub type Fields = Range<FieldId>;

pub const NO_FIELDS: Fields = Range {
    start: FieldId(0),
    end: FieldId(0),
};

impl From<super::v2::FederatedGraphV2> for FederatedGraphV3 {
    fn from(value: super::v2::FederatedGraphV2) -> Self {
        let fields = value
            .fields
            .iter()
            .map(|field| Field {
                name: field.name,
                r#type: (&value[field.field_type_id]).into(),
                arguments: field.arguments,
                resolvable_in: field.resolvable_in.clone(),
                provides: field.provides.clone(),
                requires: field.requires.clone(),
                overrides: field.overrides.clone(),
                composed_directives: field.composed_directives,
                description: field.description,
            })
            .collect();

        let input_value_definitions = value
            .input_value_definitions
            .iter()
            .map(|input_value| InputValueDefinition {
                name: input_value.name,
                r#type: (&value[input_value.type_id]).into(),
                directives: input_value.directives,
                description: input_value.description,
            })
            .collect();

        let mut object_fields: Vec<Option<Fields>> = vec![None; value.objects.len()];
        let mut interface_fields: Vec<Option<Fields>> = vec![None; value.interfaces.len()];

        for object_field in value.object_fields {
            match &mut object_fields[object_field.object_id.0] {
                entry @ None => {
                    *entry = Some(Range {
                        start: object_field.field_id,
                        end: FieldId(object_field.field_id.0 + 1),
                    });
                }
                Some(fields) => {
                    fields.end = FieldId(object_field.field_id.0 + 1);
                }
            }
        }

        for interface_field in value.interface_fields {
            match &mut interface_fields[interface_field.interface_id.0] {
                entry @ None => {
                    *entry = Some(Range {
                        start: interface_field.field_id,
                        end: FieldId(interface_field.field_id.0 + 1),
                    });
                }
                Some(fields) => {
                    fields.end = FieldId(interface_field.field_id.0 + 1);
                }
            }
        }

        FederatedGraphV3 {
            subgraphs: value.subgraphs,
            root_operation_types: value.root_operation_types,
            objects: value
                .objects
                .into_iter()
                .enumerate()
                .map(|(idx, object)| Object {
                    name: object.name,
                    implements_interfaces: object.implements_interfaces,
                    keys: object.keys,
                    composed_directives: object.composed_directives,
                    fields: object_fields[idx].as_ref().cloned().unwrap_or(NO_FIELDS),
                    description: object.description,
                })
                .collect(),
            interfaces: value
                .interfaces
                .into_iter()
                .enumerate()
                .map(|(idx, iface)| Interface {
                    name: iface.name,
                    implements_interfaces: iface.implements_interfaces,
                    keys: iface.keys,
                    composed_directives: iface.composed_directives,
                    fields: interface_fields[idx].as_ref().cloned().unwrap_or(NO_FIELDS),
                    description: iface.description,
                })
                .collect(),
            fields,
            enums: value.enums,
            unions: value.unions,
            scalars: value.scalars,
            input_objects: value.input_objects,
            enum_values: value.enum_values,
            input_value_definitions,
            strings: value.strings,
            directives: value
                .directives
                .into_iter()
                .map(|directive| match directive {
                    super::v2::Directive::Inaccessible => Directive::Inaccessible,
                    super::v2::Directive::Deprecated { reason } => Directive::Deprecated { reason },
                    super::v2::Directive::Other { name, arguments } => Directive::Other { name, arguments },
                })
                .collect(),
        }
    }
}

impl From<&super::v2::FieldType> for Type {
    fn from(value: &super::v2::FieldType) -> Self {
        let mut wrapping = Wrapping::new(value.inner_is_required);

        for wrapper in &value.list_wrappers {
            wrapping = match wrapper {
                super::v2::ListWrapper::RequiredList => wrapping.wrapped_by_required_list(),
                super::v2::ListWrapper::NullableList => wrapping.wrapped_by_nullable_list(),
            }
        }

        Type {
            wrapping,
            definition: value.kind,
        }
    }
}

impl std::ops::Index<Directives> for FederatedGraphV3 {
    type Output = [Directive];

    fn index(&self, index: Directives) -> &Self::Output {
        let (DirectiveId(start), len) = index;
        &self.directives[start..(start + len)]
    }
}

impl std::ops::Index<InputValueDefinitions> for FederatedGraphV3 {
    type Output = [InputValueDefinition];

    fn index(&self, index: InputValueDefinitions) -> &Self::Output {
        let (InputValueDefinitionId(start), len) = index;
        &self.input_value_definitions[start..(start + len)]
    }
}

impl std::ops::Index<EnumValues> for FederatedGraphV3 {
    type Output = [EnumValue];

    fn index(&self, index: EnumValues) -> &Self::Output {
        let (EnumValueId(start), len) = index;
        &self.enum_values[start..(start + len)]
    }
}

impl std::ops::Index<Fields> for FederatedGraphV3 {
    type Output = [Field];

    fn index(&self, index: Fields) -> &Self::Output {
        let Range {
            start: FieldId(start),
            end: FieldId(end),
        } = index;
        &self.fields[start..end]
    }
}

macro_rules! id_newtypes {
    ($($name:ident + $storage:ident + $out:ident,)*) => {
        $(
            impl std::ops::Index<$name> for FederatedGraphV3 {
                type Output = $out;

                fn index(&self, index: $name) -> &$out {
                    &self.$storage[index.0]
                }
            }

            impl std::ops::IndexMut<$name> for FederatedGraphV3 {
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
    InputValueDefinitionId + input_value_definitions + InputValueDefinition,
    InputObjectId + input_objects + InputObject,
    InterfaceId + interfaces + Interface,
    ObjectId + objects + Object,
    ScalarId + scalars + Scalar,
    StringId + strings + String,
    SubgraphId + subgraphs + Subgraph,
    UnionId + unions + Union,
}
