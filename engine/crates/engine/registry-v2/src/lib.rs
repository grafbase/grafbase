use std::{
    cmp::Ordering,
    collections::{BTreeMap, HashMap},
};

use gateway_v2_auth_config::v1::AuthConfig;
use ids::{MetaDirectiveId, MetaFieldId, MetaTypeId, StringId};
use indexmap::IndexSet;
use postgres_connector_types::database_definition::DatabaseDefinition;

mod common;
mod cors;
mod extensions;
mod federation_entity;
mod field_types;
mod generated;
mod misc_types;
mod operation_limits;
mod trusted_docs;

pub mod cache_control;
pub mod ids;
pub mod mongodb;
pub mod resolvers;
pub mod validators;
pub mod writer;

pub use self::{
    cache_control::CacheControl,
    common::*,
    cors::CorsConfig,
    federation_entity::*,
    field_types::{MetaFieldType, MetaInputValueType},
    generated::{
        directives::MetaDirective,
        enums::{EnumType, MetaEnumValue},
        field::MetaField,
        inputs::{InputObjectType, MetaInputValue},
        interface::InterfaceType,
        metatype::MetaType,
        objects::ObjectType,
        scalar::ScalarType,
        union::UnionType,
    },
    misc_types::*,
    mongodb::MongoDBConfiguration,
    operation_limits::*,
    trusted_docs::*,
};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Registry {
    strings: IndexSet<Box<str>>,

    types: Vec<storage::MetaTypeRecord>,

    objects: Vec<storage::ObjectTypeRecord>,
    object_fields: Vec<storage::MetaFieldRecord>,

    input_objects: Vec<storage::InputObjectTypeRecord>,
    input_values: Vec<storage::MetaInputValueRecord>,
    input_validators: Vec<storage::InputValidatorRecord>,

    enums: Vec<storage::EnumTypeRecord>,
    enum_values: Vec<storage::MetaEnumValueRecord>,

    interfaces: Vec<storage::InterfaceTypeRecord>,
    scalars: Vec<storage::ScalarTypeRecord>,
    unions: Vec<storage::UnionTypeRecord>,

    directives: Vec<storage::MetaDirectiveRecord>,

    typename_index: MetaFieldId,

    implements: HashMap<MetaTypeId, Vec<MetaTypeId>>,
    query_type: MetaTypeId,
    mutation_type: Option<MetaTypeId>,
    subscription_type: Option<MetaTypeId>,
    pub disable_introspection: bool,
    pub enable_federation: bool,
    pub federation_subscription: bool,

    pub auth: AuthConfig,
    // #[serde(default)]
    pub mongodb_configurations: HashMap<String, MongoDBConfiguration>,
    // #[serde(default)]
    pub http_headers: BTreeMap<String, ConnectorHeaders>,
    // #[serde(default)]
    pub postgres_databases: HashMap<String, DatabaseDefinition>,
    // #[serde(default)]
    pub enable_caching: bool,
    // #[serde(default)]
    pub enable_kv: bool,
    // #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub federation_entities: BTreeMap<String, FederationEntity>,
    // #[serde(default)]
    pub enable_codegen: bool,
    // FIXME: Make an enum.
    pub is_federated: bool,
    // #[serde(default)]
    pub operation_limits: OperationLimits,
    // #[serde(default)]
    pub trusted_documents: Option<TrustedDocuments>,
    // #[serde(default)]
    pub cors_config: Option<CorsConfig>,
}

impl Registry {
    pub fn types(&self) -> Iter<'_, MetaType<'_>> {
        Iter::new(
            IdRange::new(MetaTypeId::new(0), MetaTypeId::new(self.types.len())),
            self,
        )
    }

    pub fn directives(&self) -> Iter<'_, MetaDirective<'_>> {
        Iter::new(
            IdRange::new(MetaDirectiveId::new(0), MetaDirectiveId::new(self.directives.len())),
            self,
        )
    }

    pub fn lookup_type<'a>(&'a self, name: &str) -> Option<MetaType<'a>> {
        Some(self.read(self.lookup_type_id(name)?))
    }

    pub fn lookup_directive<'a>(&'a self, name: &str) -> Option<MetaDirective<'a>> {
        // This isn't that efficient, but I'm assuming small numbers of directives
        self.directives().find(|directive| directive.name() == name)
    }

    pub fn query_type(&self) -> MetaType<'_> {
        self.read(self.query_type)
    }

    pub fn mutation_type(&self) -> Option<MetaType<'_>> {
        self.mutation_type.map(|id| self.read(id))
    }

    pub fn subscription_type(&self) -> Option<MetaType<'_>> {
        self.subscription_type.map(|id| self.read(id))
    }

    pub fn root_type(&self, operation_type: OperationType) -> Option<MetaType<'_>> {
        match operation_type {
            OperationType::Query => Some(self.query_type()),
            OperationType::Mutation => self.mutation_type(),
            OperationType::Subscription => self.subscription_type(),
        }
    }

    pub fn interfaces_implemented<'a>(&'a self, name: &str) -> impl ExactSizeIterator<Item = MetaType<'a>> {
        self.lookup_type_id(name)
            .and_then(|type_id| self.implements.get(&type_id))
            .map(|v| v.as_slice())
            .unwrap_or(&[])
            .iter()
            .map(|id| self.read(*id))
    }

    pub fn has_entities(&self) -> bool {
        !self.federation_entities.is_empty()
    }

    fn lookup_type_id(&self, name: &str) -> Option<MetaTypeId> {
        let string_id = StringId::new(self.strings.get_index_of(name)?);
        let type_id = self
            .types
            .binary_search_by_key(&string_id, |ty| match ty {
                storage::MetaTypeRecord::Object(id) => self.lookup(*id).name,
                storage::MetaTypeRecord::Interface(id) => self.lookup(*id).name,
                storage::MetaTypeRecord::Union(id) => self.lookup(*id).name,
                storage::MetaTypeRecord::Enum(id) => self.lookup(*id).name,
                storage::MetaTypeRecord::InputObject(id) => self.lookup(*id).name,
                storage::MetaTypeRecord::Scalar(id) => self.lookup(*id).name,
            })
            .ok()?;

        Some(MetaTypeId::new(type_id))
    }

    pub(crate) fn string_cmp(&self, lhs: StringId, rhs: &str) -> Ordering {
        self.strings.get_index(lhs.to_index()).unwrap().as_ref().cmp(rhs)
    }
}

pub trait RegistryId: Copy {
    type Reader<'a>: From<ReadContext<'a, Self>>;

    fn read(self, ast: &Registry) -> Self::Reader<'_> {
        ReadContext {
            id: self,
            registry: ast,
        }
        .into()
    }
}

#[derive(Clone, Copy)]
pub struct ReadContext<'a, I> {
    id: I,
    registry: &'a Registry,
}

impl Registry {
    pub fn read<T>(&self, id: T) -> T::Reader<'_>
    where
        T: RegistryId,
    {
        ReadContext { id, registry: self }.into()
    }
}

trait RecordLookup<Id> {
    type Output: ?Sized;

    fn lookup(&self, index: Id) -> &Self::Output;
    // fn lookup_mut(&mut self, index: Id) -> &mut Self::Output;
}

/// Convenience module for writing to the registry.
///
/// Generally you don't need these types, but if you need one you probably need several (or all) of
/// them, so having them exposed as a single module makes them just a `use storage::*` away.
pub mod storage {
    pub use super::{
        field_types::{MetaFieldTypeRecord, MetaInputValueTypeRecord},
        generated::{
            directives::MetaDirectiveRecord,
            enums::{EnumTypeRecord, MetaEnumValueRecord},
            field::MetaFieldRecord,
            inputs::{InputObjectTypeRecord, InputValidatorRecord, MetaInputValueRecord},
            interface::InterfaceTypeRecord,
            metatype::MetaTypeRecord,
            objects::ObjectTypeRecord,
            scalar::ScalarTypeRecord,
            union::UnionTypeRecord,
        },
    };
}

// Hacky trait used by serialzation code
trait Container {
    fn is_empty(&self) -> bool;
}

impl<T> Container for IdRange<T>
where
    T: PartialEq,
{
    fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

impl<T> Container for Vec<T> {
    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}

pub fn is_false(value: &bool) -> bool {
    !value
}

#[cfg(test)]
mod tests {
    use crate::resolvers::Resolver;

    use super::*;

    #[test]
    fn types_should_have_reasonable_sizes() {
        // We do some testing on the exact size of these.
        // If the size goes up think very carefully about it.
        // If it goes down - yay, just update the test so we can keep the new low water mark.

        assert_eq!(std::mem::size_of::<CacheControl>(), 80);

        assert_eq!(std::mem::size_of::<Resolver>(), 56);
    }
}
