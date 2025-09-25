//! A structured representation of a federated GraphQL schema. Can be instantiated by [composition](https://crates.io/crates/graphql-composition) or [from SDL](`from_sdl()`).

#![expect(missing_docs)]

mod debug;
mod directive_definitions;
mod directives;
mod entity;
mod enum_definitions;
mod enum_values;
mod extensions;
mod from_sdl;
mod ids;
mod input_value_definitions;
mod objects;
mod render_sdl;
mod roots;
mod scalar_definitions;
mod r#type;
mod view;

pub use self::{
    from_sdl::DomainError,
    render_sdl::{render_api_sdl, render_federated_sdl},
};

pub(crate) use self::{
    directive_definitions::*,
    directives::*,
    entity::*,
    enum_definitions::EnumDefinitionRecord,
    enum_values::EnumValueRecord,
    extensions::*,
    ids::*,
    render_sdl::display_graphql_string_literal,
    roots::*,
    scalar_definitions::ScalarDefinitionRecord,
    r#type::{Definition, Type},
    view::{View, ViewNested},
};

use enum_definitions::EnumDefinition;
use scalar_definitions::ScalarDefinition;
use std::{fmt, ops::Range};

#[derive(Default)]
pub struct FederatedGraph {
    pub(crate) subgraphs: Vec<Subgraph>,
    pub extensions: Vec<Extension>,
    pub(crate) objects: Vec<Object>,
    pub(crate) interfaces: Vec<Interface>,
    pub(crate) fields: Vec<Field>,
    pub(crate) roots: SchemaRoots,

    pub(crate) directive_definitions: Vec<DirectiveDefinitionRecord>,
    pub(crate) directive_definition_arguments: Vec<DirectiveDefinitionArgument>,
    pub(crate) scalar_definitions: Vec<ScalarDefinitionRecord>,
    pub(crate) enum_definitions: Vec<EnumDefinitionRecord>,
    pub(crate) unions: Vec<Union>,
    pub(crate) input_objects: Vec<InputObject>,
    pub(crate) enum_values: Vec<EnumValueRecord>,
    pub(crate) linked_schemas: Vec<LinkDirective>,
    pub(crate) composed_directives_on_schema_definition: Vec<directives::OtherDirective>,

    /// All [input value definitions](http://spec.graphql.org/October2021/#InputValueDefinition) in the federated graph. Concretely, these are arguments of output fields, and input object fields.
    pub(crate) input_value_definitions: Vec<InputValueDefinition>,

    /// All the strings in the federated graph, deduplicated.
    pub strings: Vec<String>,
}

impl FederatedGraph {
    pub fn from_sdl(sdl: &str) -> Result<Self, crate::DomainError> {
        if sdl.trim().is_empty() {
            return Ok(Default::default());
        }
        from_sdl::from_sdl(sdl)
    }

    pub(crate) fn definition_name(&self, definition: Definition) -> &str {
        let name_id = match definition {
            Definition::Scalar(scalar_id) => self[scalar_id].name,
            Definition::Object(object_id) => self.at(object_id).name,
            Definition::Interface(interface_id) => self.at(interface_id).name,
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

    pub fn iter_scalar_definitions(&self) -> impl Iterator<Item = ScalarDefinition<'_>> {
        self.scalar_definitions
            .iter()
            .enumerate()
            .map(|(idx, _)| self.at(ScalarDefinitionId::from(idx)))
    }

    pub fn iter_enum_definitions(&self) -> impl Iterator<Item = EnumDefinition<'_>> {
        self.enum_definitions
            .iter()
            .enumerate()
            .map(|(idx, _)| self.at(EnumDefinitionId::from(idx)))
    }
}

#[derive(Clone, Debug)]
pub struct Subgraph {
    pub name: StringId,
    pub join_graph_enum_value: EnumValueId,
    pub url: Option<StringId>,
}

#[derive(Clone, Debug)]
pub struct Union {
    pub(crate) name: StringId,
    pub(crate) description: Option<StringId>,
    pub(crate) members: Vec<ObjectId>,
    pub(crate) directives: Vec<Directive>,
}

#[derive(Clone, Debug)]
pub struct InputObject {
    pub(crate) name: StringId,
    pub(crate) description: Option<StringId>,
    pub(crate) fields: InputValueDefinitions,
    pub(crate) directives: Vec<Directive>,
}

#[derive(Default, Clone, PartialEq, PartialOrd, Debug)]
#[allow(clippy::enum_variant_names)]
pub(crate) enum Value {
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

#[derive(Clone, Debug)]
pub struct Object {
    pub(crate) name: StringId,
    pub(crate) directives: Vec<Directive>,
    pub(crate) description: Option<StringId>,
    pub(crate) implements_interfaces: Vec<InterfaceId>,
    pub(crate) fields: Fields,
}

#[derive(Clone, Debug)]
pub struct Interface {
    pub(crate) name: StringId,
    pub(crate) directives: Vec<Directive>,
    pub(crate) description: Option<StringId>,
    pub(crate) implements_interfaces: Vec<InterfaceId>,
    pub(crate) fields: Fields,
}

#[derive(Clone, Debug)]
pub struct Field {
    pub(crate) parent_entity_id: EntityDefinitionId,
    pub(crate) name: StringId,
    pub(crate) description: Option<StringId>,
    pub(crate) r#type: Type,
    pub(crate) arguments: InputValueDefinitions,
    pub(crate) directives: Vec<Directive>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct InputValueDefinition {
    pub(crate) name: StringId,
    pub(crate) r#type: Type,
    pub(crate) directives: Vec<Directive>,
    pub(crate) description: Option<StringId>,
    pub(crate) default: Option<Value>,
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub(crate) struct SelectionSet(pub(crate) Vec<Selection>);

impl From<Vec<Selection>> for SelectionSet {
    fn from(selections: Vec<Selection>) -> Self {
        SelectionSet(selections)
    }
}

impl FromIterator<Selection> for SelectionSet {
    fn from_iter<I: IntoIterator<Item = Selection>>(iter: I) -> Self {
        SelectionSet(iter.into_iter().collect())
    }
}

impl std::ops::Deref for SelectionSet {
    type Target = Vec<Selection>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for SelectionSet {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub(crate) enum Selection {
    Typename,
    Field(FieldSelection),
    InlineFragment { on: Definition, subselection: SelectionSet },
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct FieldSelection {
    pub field_id: FieldId,
    pub arguments: Vec<(InputValueDefinitionId, Value)>,
    pub subselection: SelectionSet,
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
pub type InputValueDefinitions = (InputValueDefinitionId, usize);

pub const NO_INPUT_VALUE_DEFINITION: InputValueDefinitions = (InputValueDefinitionId::const_from_usize(0), 0);
pub const NO_FIELDS: Fields = Range {
    start: FieldId::const_from_usize(0),
    end: FieldId::const_from_usize(0),
};
