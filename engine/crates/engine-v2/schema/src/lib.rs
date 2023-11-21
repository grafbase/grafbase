#![allow(dead_code)]

use std::{borrow::Cow, cmp::Ordering};

mod conversion;
mod ids;
pub mod introspection;
mod names;
mod walkers;

pub use ids::*;
use introspection::{IntrospectionDataSource, IntrospectionResolver};
pub use names::Names;
pub use walkers::*;

/// This does NOT need to be backwards compatible. We'll probably cache it for performance, but it is not
/// the source of truth. If the cache is stale we would just re-create this Graph from its source:
/// federated_graph::FederatedGraph.
pub struct Schema {
    pub description: Option<StringId>,
    data_sources: Vec<DataSource>,
    subgraphs: Vec<Subgraph>,

    pub root_operation_types: RootOperationTypes,
    objects: Vec<Object>,
    // Sorted by object_id, field name (actual string)
    object_fields: Vec<ObjectField>,

    interfaces: Vec<Interface>,
    // Sorted by interface_id, field name (actual string)
    interface_fields: Vec<InterfaceField>,

    fields: Vec<Field>,

    enums: Vec<Enum>,
    unions: Vec<Union>,
    scalars: Vec<Scalar>,
    input_objects: Vec<InputObject>,
    input_values: Vec<InputValue>,
    resolvers: Vec<Resolver>,

    /// All the strings in the supergraph, deduplicated.
    strings: Vec<String>,
    /// All the field types in the supergraph, deduplicated.
    types: Vec<Type>,
    // All definitions sorted by their name (actual string)
    pub definitions: Vec<Definition>,
}

impl Schema {
    pub fn definition_by_name(&self, name: &str) -> Option<Definition> {
        self.definitions
            .binary_search_by_key(&name, |definition| self.definition_name(*definition))
            .map(|index| self.definitions[index])
            .ok()
    }

    pub fn object_field_by_name(&self, object_id: ObjectId, name: &str) -> Option<FieldId> {
        self.object_fields
            .binary_search_by_key(&(object_id, name), |ObjectField { object_id, field_id }| {
                (*object_id, &self[self[*field_id].name])
            })
            .map(|index| self.object_fields[index].field_id)
            .ok()
    }

    pub fn interface_field_by_name(&self, interface_id: InterfaceId, name: &str) -> Option<FieldId> {
        self.interface_fields
            .binary_search_by_key(&(interface_id, name), |InterfaceField { interface_id, field_id }| {
                (*interface_id, &self[self[*field_id].name])
            })
            .map(|index| self.interface_fields[index].field_id)
            .ok()
    }

    fn ensure_proper_ordering(&mut self) {
        let mut object_fields = std::mem::take(&mut self.object_fields);
        object_fields
            .sort_unstable_by_key(|ObjectField { object_id, field_id }| (*object_id, &self[self[*field_id].name]));
        self.object_fields = object_fields;

        let mut interface_fields = std::mem::take(&mut self.interface_fields);
        interface_fields.sort_unstable_by_key(|InterfaceField { interface_id, field_id }| {
            (*interface_id, &self[self[*field_id].name])
        });
        self.interface_fields = interface_fields;

        let mut definitions = std::mem::take(&mut self.definitions);
        definitions.sort_unstable_by_key(|definition| self.definition_name(*definition));
        self.definitions = definitions;
    }

    fn definition_name(&self, definition: Definition) -> &str {
        let name = match definition {
            Definition::Scalar(s) => self[s].name,
            Definition::Object(o) => self[o].name,
            Definition::Interface(i) => self[i].name,
            Definition::Union(u) => self[u].name,
            Definition::Enum(e) => self[e].name,
            Definition::InputObject(io) => self[io].name,
        };
        &self[name]
    }
}

#[derive(Debug)]
pub struct RootOperationTypes {
    pub query: ObjectId,
    pub mutation: Option<ObjectId>,
    pub subscription: Option<ObjectId>,
}

impl std::fmt::Debug for Schema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(std::any::type_name::<Schema>()).finish_non_exhaustive()
    }
}

pub enum DataSource {
    Introspection(Box<IntrospectionDataSource>),
    Subgraph(SubgraphId),
}

impl DataSource {
    pub fn as_introspection(&self) -> Option<&IntrospectionDataSource> {
        match self {
            DataSource::Introspection(introspection) => Some(introspection),
            DataSource::Subgraph(_) => None,
        }
    }
}

#[derive(Debug)]
pub struct Subgraph {
    pub name: StringId,
    pub url: StringId,
}

#[derive(Debug)]
pub struct Object {
    pub name: StringId,
    pub description: Option<StringId>,
    pub interfaces: Vec<InterfaceId>,
    /// All _resolvable_ keys.
    resolvable_keys: Vec<Key>,
    /// All directives that made it through composition. Notably includes `@tag`.
    pub composed_directives: Vec<Directive>,
}

#[derive(PartialOrd, Ord, PartialEq, Eq)]
pub struct ObjectField {
    pub object_id: ObjectId,
    pub field_id: FieldId,
}

#[derive(Debug)]
pub struct Key {
    /// The subgraph that can resolve the entity with these fields.
    pub subgraph_id: SubgraphId,

    /// Corresponds to the fields in an `@key` directive.
    pub fields: FieldSet,
}

#[derive(Debug, Default, Clone)]
pub struct FieldSet {
    // sorted by field id
    items: Vec<FieldSetItem>,
}

impl FromIterator<FieldSetItem> for FieldSet {
    fn from_iter<T: IntoIterator<Item = FieldSetItem>>(iter: T) -> Self {
        let mut items = iter.into_iter().collect::<Vec<_>>();
        items.sort_unstable_by_key(|selection| selection.field);
        Self { items }
    }
}

impl IntoIterator for FieldSet {
    type Item = FieldSetItem;

    type IntoIter = <Vec<FieldSetItem> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

impl<'a> IntoIterator for &'a FieldSet {
    type Item = &'a FieldSetItem;

    type IntoIter = <&'a Vec<FieldSetItem> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}

impl FieldSet {
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &FieldSetItem> + '_ {
        self.items.iter()
    }

    pub fn selection(&self, field: FieldId) -> Option<&FieldSetItem> {
        let index = self
            .items
            .binary_search_by_key(&field, |selection| selection.field)
            .ok()?;
        Some(&self.items[index])
    }

    pub fn merge_opt(left_set: Option<&FieldSet>, right_set: Option<&FieldSet>) -> FieldSet {
        match (left_set, right_set) {
            (Some(left_set), Some(right_set)) => Self::merge(left_set, right_set),
            (Some(left_set), None) => left_set.clone(),
            (None, Some(right_set)) => right_set.clone(),
            (None, None) => FieldSet::default(),
        }
    }

    pub fn merge(left_set: &FieldSet, right_set: &FieldSet) -> FieldSet {
        let mut items = vec![];
        let mut l = 0;
        let mut r = 0;
        while l < left_set.items.len() && r < right_set.items.len() {
            let left = &left_set.items[l];
            let right = &right_set.items[r];
            match left.field.cmp(&right.field) {
                Ordering::Less => {
                    items.push(left.clone());
                    l += 1;
                }
                Ordering::Equal => {
                    items.push(right.clone());
                    r += 1;
                }
                Ordering::Greater => {
                    items.push(FieldSetItem {
                        field: left.field,
                        subselection: Self::merge(&left.subselection, &right.subselection),
                    });
                    l += 1;
                    r += 1;
                }
            }
        }
        FieldSet { items }
    }
}

#[derive(Debug, Clone)]
pub struct FieldSetItem {
    pub field: FieldId,
    pub subselection: FieldSet,
}

#[derive(Debug)]
pub struct Field {
    pub name: StringId,
    pub description: Option<StringId>,
    pub type_id: TypeId,
    pub resolvers: Vec<FieldResolver>,
    pub is_deprecated: bool,
    pub deprecated_reason: Option<StringId>,

    /// Special case when only going through this field children are accessible.
    provides: Vec<FieldProvides>,

    pub arguments: Vec<InputValueId>,

    /// All directives that made it through composition. Notably includes `@tag`.
    pub composed_directives: Vec<Directive>,
}

impl Field {
    pub fn provides(&self, data_source_id: DataSourceId) -> Option<&FieldSet> {
        self.provides
            .iter()
            .find(|provides| provides.data_source_id == data_source_id)
            .map(|provides| &provides.fields)
    }
}

#[derive(Debug)]
pub struct FieldResolver {
    pub resolver_id: ResolverId,
    pub requires: FieldSet,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Resolver {
    Introspection(IntrospectionResolver),
    Subgraph(SubgraphResolver),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SubgraphResolver {
    pub subgraph_id: SubgraphId,
}

impl Resolver {
    pub fn data_source_id(&self) -> DataSourceId {
        match self {
            Resolver::Subgraph(resolver) => DataSourceId::from(resolver.subgraph_id),
            Resolver::Introspection(resolver) => resolver.data_source_id,
        }
    }

    pub fn requires(&self) -> Cow<'_, FieldSet> {
        Cow::Owned(FieldSet::default())
    }
}

#[derive(Debug)]
pub struct Directive {
    pub name: StringId,
    pub arguments: Vec<(StringId, Value)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    String(StringId),
    Int(i64),
    Float(StringId),
    Boolean(bool),
    EnumValue(StringId),
    Object(Vec<(StringId, Value)>),
    List(Vec<Value>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Definition {
    Scalar(ScalarId),
    Object(ObjectId),
    Interface(InterfaceId),
    Union(UnionId),
    Enum(EnumId),
    InputObject(InputObjectId),
}

impl From<ScalarId> for Definition {
    fn from(id: ScalarId) -> Self {
        Self::Scalar(id)
    }
}

impl From<ObjectId> for Definition {
    fn from(id: ObjectId) -> Self {
        Self::Object(id)
    }
}

impl From<InterfaceId> for Definition {
    fn from(id: InterfaceId) -> Self {
        Self::Interface(id)
    }
}

impl From<UnionId> for Definition {
    fn from(id: UnionId) -> Self {
        Self::Union(id)
    }
}

impl From<EnumId> for Definition {
    fn from(id: EnumId) -> Self {
        Self::Enum(id)
    }
}

impl From<InputObjectId> for Definition {
    fn from(id: InputObjectId) -> Self {
        Self::InputObject(id)
    }
}

#[derive(Debug)]
pub struct Type {
    pub inner: Definition,
    pub wrapping: Wrapping,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Wrapping {
    /// Is the innermost type required?
    ///
    /// Examples:
    ///
    /// - `String` => false
    /// - `String!` => true
    /// - `[String!]` => true
    /// - `[String]!` => false
    pub inner_is_required: bool,

    /// Innermost to outermost.
    pub list_wrapping: Vec<ListWrapping>,
}

impl Wrapping {
    fn nullable() -> Self {
        Wrapping {
            inner_is_required: false,
            list_wrapping: vec![],
        }
    }

    fn required() -> Self {
        Wrapping {
            inner_is_required: true,
            list_wrapping: vec![],
        }
    }

    fn nullable_list(self) -> Self {
        Wrapping {
            list_wrapping: [ListWrapping::NullableList]
                .into_iter()
                .chain(self.list_wrapping)
                .collect(),
            ..self
        }
    }

    fn required_list(self) -> Self {
        Wrapping {
            list_wrapping: [ListWrapping::RequiredList]
                .into_iter()
                .chain(self.list_wrapping)
                .collect(),
            ..self
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListWrapping {
    RequiredList,
    NullableList,
}

/// Represents an `@provides` directive on a field in a subgraph.
#[derive(Debug)]
pub struct FieldProvides {
    pub data_source_id: DataSourceId,
    pub fields: FieldSet,
}

#[derive(Debug)]
pub struct Interface {
    pub name: StringId,
    pub description: Option<StringId>,
    pub interfaces: Vec<InterfaceId>,

    pub possible_types: Vec<ObjectId>,

    /// All directives that made it through composition. Notably includes `@tag`.
    pub composed_directives: Vec<Directive>,
}

#[derive(Debug)]
pub struct InterfaceField {
    pub interface_id: InterfaceId,
    pub field_id: FieldId,
}

#[derive(Debug)]
pub struct Enum {
    pub name: StringId,
    pub description: Option<StringId>,
    pub values: Vec<EnumValue>,

    /// All directives that made it through composition. Notably includes `@tag`.
    pub composed_directives: Vec<Directive>,
}

#[derive(Debug)]
pub struct EnumValue {
    pub name: StringId,
    pub description: Option<StringId>,
    pub is_deprecated: bool,
    pub deprecated_reason: Option<StringId>,

    /// All directives that made it through composition. Notably includes `@tag`.
    pub composed_directives: Vec<Directive>,
}

#[derive(Debug)]
pub struct Union {
    pub name: StringId,
    pub description: Option<StringId>,
    pub possible_types: Vec<ObjectId>,

    /// All directives that made it through composition. Notably includes `@tag`.
    pub composed_directives: Vec<Directive>,
}

#[derive(Debug)]
pub struct Scalar {
    pub name: StringId,
    pub description: Option<StringId>,
    pub specified_by_url: Option<StringId>,
    /// All directives that made it through composition. Notably includes `@tag`.
    pub composed_directives: Vec<Directive>,
}

#[derive(Debug)]
pub struct InputObject {
    pub name: StringId,
    pub description: Option<StringId>,
    pub input_fields: Vec<InputValueId>,

    /// All directives that made it through composition. Notably includes `@tag`.
    pub composed_directives: Vec<Directive>,
}

#[derive(Debug, Clone)]
pub struct InputValue {
    pub name: StringId,
    pub description: Option<StringId>,
    pub type_id: TypeId,
    pub default_value: Option<Value>,
}

impl Schema {
    pub fn default_walker(&self) -> SchemaWalker<'_, ()> {
        self.walker(self)
    }

    pub fn walker<'a>(&'a self, names: &'a dyn Names) -> SchemaWalker<'a, ()> {
        SchemaWalker::new((), self, names)
    }
}
