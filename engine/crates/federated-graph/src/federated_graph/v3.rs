use std::ops::Range;

pub use super::v2::{
    Definition, DirectiveId, Directives, Enum, EnumId, EnumValue, EnumValueId, EnumValues, FieldId, FieldProvides,
    FieldRequires, FieldSet, FieldSetItem, InputObject, InputObjectId, InputValueDefinitionId, InputValueDefinitions,
    InterfaceId, Key, ObjectId, Override, OverrideLabel, OverrideSource, RootOperationTypes, Scalar, ScalarId,
    StringId, Subgraph, SubgraphId, Union, UnionId, Value, NO_DIRECTIVES, NO_ENUM_VALUE, NO_INPUT_VALUE_DEFINITION,
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

    /// All @authorized directives
    #[serde(default)]
    pub authorized_directives: Vec<AuthorizedDirective>,
    #[serde(default)]
    pub field_authorized_directives: Vec<(FieldId, AuthorizedDirectiveId)>,
    #[serde(default)]
    pub object_authorized_directives: Vec<(ObjectId, AuthorizedDirectiveId)>,
}

impl FederatedGraphV3 {
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
}

impl std::fmt::Debug for FederatedGraphV3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(std::any::type_name::<Self>()).finish_non_exhaustive()
    }
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

#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq, PartialOrd)]
pub struct AuthorizedDirective {
    pub fields: Option<FieldSet>,
    pub arguments: Option<InputValueDefinitionSet>,
    pub metadata: Option<Value>,
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

pub type InputValueDefinitionSet = Vec<InputValueDefinitionSetItem>;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, PartialOrd)]
pub struct InputValueDefinitionSetItem {
    pub input_value_definition: InputValueDefinitionId,
    pub subselection: InputValueDefinitionSet,
}

/// A (start, end) range in FederatedGraph::fields.
pub type Fields = Range<FieldId>;

pub const NO_FIELDS: Fields = Range {
    start: FieldId(0),
    end: FieldId(0),
};

impl From<super::v2::FederatedGraphV2> for FederatedGraphV3 {
    fn from(mut value: super::v2::FederatedGraphV2) -> Self {
        let mut fields: Vec<Field> = value
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

        // In FederatedGraphV3, we reserve two fields for __schema and __type
        // on the root Query type. The space for them needs to be created. This
        // block is responsible for it.
        {
            let new_fields = ["__schema", "__type"].map(|needle| {
                value
                    .strings
                    .iter()
                    .position(|string| string == needle)
                    .map(StringId)
                    .unwrap_or_else(|| {
                        let idx = value.strings.len();
                        value.strings.push((*needle).to_owned());
                        StringId(idx)
                    })
            });

            let query_object_id = value.root_operation_types.query;
            let original_start = {
                let query_object_fields = object_fields[query_object_id.0].as_mut().expect("Query to have fields");
                let original_start = query_object_fields.start.0;
                query_object_fields.end.0 += 2;

                original_start
            };

            fields.splice(
                original_start..original_start,
                new_fields.into_iter().map(|name| Field {
                    name,
                    r#type: Type {
                        wrapping: Wrapping::new(false),
                        definition: Definition::Object(query_object_id),
                    },
                    arguments: NO_INPUT_VALUE_DEFINITION,
                    resolvable_in: Vec::new(),
                    provides: Vec::new(),
                    requires: Vec::new(),
                    overrides: Vec::new(),
                    composed_directives: NO_DIRECTIVES,
                    description: None,
                }),
            );

            fn correct_fieldset(original_start: usize, fieldset: &mut FieldSet) {
                for item in fieldset {
                    if item.field.0 >= original_start {
                        item.field.0 += 2;
                    }
                    correct_fieldset(original_start, &mut item.subselection);
                }
            }

            for object in &mut value.objects {
                for key in &mut object.keys {
                    correct_fieldset(original_start, &mut key.fields);
                }
            }

            for interface in &mut value.interfaces {
                for key in &mut interface.keys {
                    correct_fieldset(original_start, &mut key.fields);
                }
            }

            for field in &mut fields {
                for provides in &mut field.provides {
                    correct_fieldset(original_start, &mut provides.fields);
                }
                for requires in &mut field.requires {
                    correct_fieldset(original_start, &mut requires.fields);
                }
            }
        };

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
            ..Default::default()
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

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct AuthorizedDirectiveId(pub usize);

impl From<AuthorizedDirectiveId> for usize {
    fn from(value: AuthorizedDirectiveId) -> usize {
        value.0
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

impl Default for FederatedGraphV3 {
    fn default() -> Self {
        FederatedGraphV3 {
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
        }
    }
}
