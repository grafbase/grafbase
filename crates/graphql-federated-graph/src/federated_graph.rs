mod debug;
mod directives;
mod enum_values;
mod ids;
mod objects;
mod root_operation_types;
mod r#type;
mod type_definitions;
mod view;

pub use self::{
    directives::*,
    enum_values::{EnumValue, EnumValueRecord},
    ids::*,
    r#type::{Definition, Type},
    root_operation_types::RootOperationTypes,
    type_definitions::{TypeDefinition, TypeDefinitionKind, TypeDefinitionRecord},
    view::{View, ViewNested},
};
pub use wrapping::Wrapping;

use std::{collections::BTreeSet, ops::Range};

#[derive(Clone)]
pub struct FederatedGraph {
    pub subgraphs: Vec<Subgraph>,
    pub root_operation_types: RootOperationTypes,
    pub type_definitions: Vec<TypeDefinitionRecord>,
    pub objects: Vec<Object>,
    pub interfaces: Vec<Interface>,
    pub fields: Vec<Field>,

    pub unions: Vec<Union>,
    pub input_objects: Vec<InputObject>,
    pub enum_values: Vec<EnumValueRecord>,

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

    pub list_sizes: Vec<(FieldId, ListSize)>,
}

impl FederatedGraph {
    /// Instantiate a [FederatedGraph] from a federated schema string.
    #[cfg(feature = "from_sdl")]
    pub fn from_sdl(sdl: &str) -> Result<Self, crate::DomainError> {
        crate::from_sdl(sdl)
    }

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

#[derive(Clone)]
pub struct Subgraph {
    pub name: StringId,
    pub url: StringId,
}

#[derive(Clone)]
pub struct Union {
    pub name: StringId,
    pub members: Vec<ObjectId>,
    pub join_members: BTreeSet<(SubgraphId, ObjectId)>,

    /// All directives that made it through composition. Notably includes `@tag`.
    pub composed_directives: Directives,

    pub description: Option<StringId>,
}

#[derive(Clone)]
pub struct InputObject {
    pub name: StringId,

    pub fields: InputValueDefinitions,

    /// All directives that made it through composition. Notably includes `@tag`.
    pub composed_directives: Directives,

    pub description: Option<StringId>,
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
    Cost {
        weight: i32,
    },
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

    pub arguments: InputValueDefinitions,

    pub join_fields: Vec<JoinField>,

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

#[derive(Clone)]
pub struct JoinField {
    pub subgraph_id: SubgraphId,
    // Only present if different from the field type.
    pub r#type: Option<Type>,
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

#[derive(Clone, PartialEq)]
pub struct InputValueDefinition {
    pub name: StringId,
    pub r#type: Type,
    pub directives: Directives,
    pub description: Option<StringId>,
    pub default: Option<Value>,
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
                query: ObjectId::from(0),
                mutation: None,
                subscription: None,
            },
            objects: vec![Object {
                type_definition_id: TypeDefinitionId::from(0),
                implements_interfaces: Vec::new(),
                join_implements: Vec::new(),
                keys: Vec::new(),
                fields: FieldId::from(0)..FieldId::from(2),
            }],
            interfaces: Vec::new(),
            fields: vec![
                Field {
                    name: StringId::from(1),
                    r#type: Type {
                        wrapping: Default::default(),
                        definition: Definition::Scalar(0usize.into()),
                    },
                    join_fields: Vec::new(),
                    arguments: NO_INPUT_VALUE_DEFINITION,
                    resolvable_in: Vec::new(),
                    provides: Vec::new(),
                    requires: Vec::new(),
                    overrides: Vec::new(),
                    composed_directives: NO_DIRECTIVES,
                    description: None,
                },
                Field {
                    name: StringId::from(2),
                    r#type: Type {
                        wrapping: Default::default(),
                        definition: Definition::Scalar(0usize.into()),
                    },
                    join_fields: Vec::new(),
                    arguments: NO_INPUT_VALUE_DEFINITION,
                    resolvable_in: Vec::new(),
                    provides: Vec::new(),
                    requires: Vec::new(),
                    overrides: Vec::new(),
                    composed_directives: NO_DIRECTIVES,
                    description: None,
                },
            ],
            unions: Vec::new(),
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
            list_sizes: Vec::new(),
        }
    }
}

impl std::ops::Index<Directives> for FederatedGraph {
    type Output = [Directive];

    fn index(&self, index: Directives) -> &Self::Output {
        let (start, len) = index;
        &self.directives[usize::from(start)..(usize::from(start) + len)]
    }
}

impl std::ops::Index<InputValueDefinitions> for FederatedGraph {
    type Output = [InputValueDefinition];

    fn index(&self, index: InputValueDefinitions) -> &Self::Output {
        let (start, len) = index;
        &self.input_value_definitions[usize::from(start)..(usize::from(start) + len)]
    }
}

impl std::ops::Index<Fields> for FederatedGraph {
    type Output = [Field];

    fn index(&self, index: Fields) -> &Self::Output {
        &self.fields[usize::from(index.start)..usize::from(index.end)]
    }
}

pub type InputValueDefinitionSet = Vec<InputValueDefinitionSetItem>;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, PartialOrd)]
pub struct InputValueDefinitionSetItem {
    pub input_value_definition: InputValueDefinitionId,
    pub subselection: InputValueDefinitionSet,
}

/// A (start, end) range in FederatedGraph::fields.
pub type Fields = Range<FieldId>;
/// A (start, len) range in FederatedSchema.
pub type Directives = (DirectiveId, usize);
/// A (start, len) range in FederatedSchema.
pub type InputValueDefinitions = (InputValueDefinitionId, usize);

pub const NO_DIRECTIVES: Directives = (DirectiveId::const_from_usize(0), 0);
pub const NO_INPUT_VALUE_DEFINITION: InputValueDefinitions = (InputValueDefinitionId::const_from_usize(0), 0);
pub const NO_FIELDS: Fields = Range {
    start: FieldId::const_from_usize(0),
    end: FieldId::const_from_usize(0),
};

pub type FieldSet = Vec<FieldSetItem>;

#[derive(Clone, PartialEq, PartialOrd)]
pub struct FieldSetItem {
    pub field: FieldId,
    pub arguments: Vec<(InputValueDefinitionId, Value)>,
    pub subselection: FieldSet,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn override_label() {
        assert!("".parse::<OverrideLabel>().is_err());
        assert!("percent(heh)".parse::<OverrideLabel>().is_err());
        assert!("percent(30".parse::<OverrideLabel>().is_err());

        assert_eq!(
            "percent(30)".parse::<OverrideLabel>().unwrap().as_percent().unwrap(),
            30
        );
    }
}
