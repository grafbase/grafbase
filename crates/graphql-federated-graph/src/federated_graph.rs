mod debug;
mod directives;
mod entity;
mod enum_values;
mod ids;
mod objects;
mod root_operation_types;
mod r#type;
mod type_definitions;
mod view;

use crate::directives::*;

pub use self::{
    directives::*,
    entity::*,
    enum_values::{EnumValue, EnumValueRecord},
    ids::*,
    r#type::{Definition, Type},
    root_operation_types::RootOperationTypes,
    type_definitions::{TypeDefinition, TypeDefinitionKind, TypeDefinitionRecord},
    view::{View, ViewNested},
};
pub use wrapping::Wrapping;

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

    pub fn fields_range(&self, parent_definition_id: EntityDefinitionId) -> std::ops::Range<FieldId> {
        let start = self
            .fields
            .partition_point(|f| f.parent_entity_id < parent_definition_id);

        let end = start + self.fields[start..].partition_point(|f| f.parent_entity_id <= parent_definition_id);

        FieldId::from(start)..FieldId::from(end)
    }

    pub fn iter_interfaces(&self) -> impl ExactSizeIterator<Item = View<InterfaceId, &Interface>> {
        (0..self.interfaces.len()).map(|idx| self.view(InterfaceId::from(idx)))
    }

    pub fn iter_objects(&self) -> impl ExactSizeIterator<Item = View<ObjectId, &Object>> {
        (0..self.objects.len()).map(|idx| self.view(ObjectId::from(idx)))
    }

    pub fn iter_fields(&self, parent_definition_id: EntityDefinitionId) -> impl Iterator<Item = View<FieldId, &Field>> {
        let start = self
            .fields
            .partition_point(|f| f.parent_entity_id < parent_definition_id);

        self.fields[start..]
            .iter()
            .take_while(move |field| field.parent_entity_id == parent_definition_id)
            .enumerate()
            .map(move |(idx, field)| View {
                id: FieldId::from(start + idx),
                record: field,
            })
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
}

#[derive(Clone)]
pub struct Subgraph {
    pub name: StringId,
    pub url: StringId,
}

#[derive(Clone)]
pub struct Union {
    pub name: StringId,
    pub description: Option<StringId>,
    pub members: Vec<ObjectId>,
    pub directives: Vec<Directive>,
}

#[derive(Clone)]
pub struct InputObject {
    pub name: StringId,
    pub description: Option<StringId>,
    pub fields: InputValueDefinitions,
    pub directives: Vec<Directive>,
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
}

#[derive(Clone)]
pub struct Interface {
    pub type_definition_id: TypeDefinitionId,
    pub implements_interfaces: Vec<InterfaceId>,
}

#[derive(Clone)]
pub struct Field {
    pub parent_entity_id: EntityDefinitionId,
    pub name: StringId,
    pub description: Option<StringId>,
    pub r#type: Type,
    pub arguments: InputValueDefinitions,
    pub directives: Vec<Directive>,
}

impl Value {
    pub fn is_list(&self) -> bool {
        matches!(self, Value::List(_))
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }
}

#[derive(Clone, PartialEq)]
pub struct InputValueDefinition {
    pub name: StringId,
    pub r#type: Type,
    pub directives: Vec<Directive>,
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

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct SelectionSet(pub Vec<Selection>);

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

impl SelectionSet {
    pub fn find_field(&self, field_id: FieldId) -> Option<&FieldSelection> {
        for selection in &self.0 {
            match selection {
                Selection::Field(field) => {
                    if field.field_id == field_id {
                        return Some(field);
                    }
                }
                Selection::InlineFragment { subselection, .. } => {
                    if let Some(found) = subselection.find_field(field_id) {
                        return Some(found);
                    }
                }
            }
        }
        None
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum Selection {
    Field(FieldSelection),
    InlineFragment { on: Definition, subselection: SelectionSet },
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct FieldSelection {
    pub field_id: FieldId,
    pub arguments: Vec<(InputValueDefinitionId, Value)>,
    pub subselection: SelectionSet,
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
                directives: Vec::new(),
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
            }],
            interfaces: Vec::new(),
            fields: vec![
                Field {
                    name: StringId::from(1),
                    r#type: Type {
                        wrapping: Default::default(),
                        definition: Definition::Scalar(0usize.into()),
                    },
                    parent_entity_id: EntityDefinitionId::Object(ObjectId::from(0)),
                    arguments: NO_INPUT_VALUE_DEFINITION,
                    description: None,
                    directives: Vec::new(),
                },
                Field {
                    name: StringId::from(2),
                    r#type: Type {
                        wrapping: Default::default(),
                        definition: Definition::Scalar(0usize.into()),
                    },
                    parent_entity_id: EntityDefinitionId::Object(ObjectId::from(0)),
                    arguments: NO_INPUT_VALUE_DEFINITION,
                    description: None,
                    directives: Vec::new(),
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
        }
    }
}

impl std::ops::Index<InputValueDefinitions> for FederatedGraph {
    type Output = [InputValueDefinition];

    fn index(&self, index: InputValueDefinitions) -> &Self::Output {
        let (start, len) = index;
        &self.input_value_definitions[usize::from(start)..(usize::from(start) + len)]
    }
}

pub type InputValueDefinitionSet = Vec<InputValueDefinitionSetItem>;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, PartialOrd)]
pub struct InputValueDefinitionSetItem {
    pub input_value_definition: InputValueDefinitionId,
    pub subselection: InputValueDefinitionSet,
}

/// A (start, len) range in FederatedSchema.
pub type InputValueDefinitions = (InputValueDefinitionId, usize);

pub const NO_INPUT_VALUE_DEFINITION: InputValueDefinitions = (InputValueDefinitionId::const_from_usize(0), 0);

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
