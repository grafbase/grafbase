//! Contains a partial registry that's used by caching code in the gateway
//!
//! We don't use the full registry for this because it's large and caching
//! needs to be fast.

use std::cmp::Ordering;

use ids::{MetaTypeId, StringId};
use indexmap::IndexSet;

mod common;
mod field_types;
mod generated;

mod extensions;
pub mod ids;
pub mod writer;

pub use self::{
    common::*,
    generated::{
        field::MetaField, interface::InterfaceType, metatype::MetaType, objects::ObjectType, others::OtherType,
    },
};
pub use engine_id_newtypes::IdRange;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct PartialCacheRegistry {
    strings: IndexSet<Box<str>>,

    types: Vec<storage::MetaTypeRecord>,

    objects: Vec<storage::ObjectTypeRecord>,
    object_fields: Vec<storage::MetaFieldRecord>,

    interfaces: Vec<storage::InterfaceTypeRecord>,

    others: Vec<storage::OtherTypeRecord>,

    query_type: MetaTypeId,
    mutation_type: Option<MetaTypeId>,
    subscription_type: Option<MetaTypeId>,

    pub enable_caching: bool,
}

impl PartialCacheRegistry {
    pub fn types(&self) -> Iter<'_, MetaType<'_>> {
        Iter::new(
            IdRange::new(MetaTypeId::new(0), MetaTypeId::new(self.types.len())),
            self,
        )
    }

    pub fn lookup_type<'a>(&'a self, name: &str) -> Option<MetaType<'a>> {
        Some(self.read(self.lookup_type_id(name)?))
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

    fn lookup_type_id(&self, name: &str) -> Option<MetaTypeId> {
        let string_id = StringId::new(self.strings.get_index_of(name)?);
        let type_id = self
            .types
            .binary_search_by_key(&string_id, |ty| match ty {
                storage::MetaTypeRecord::Object(id) => self.lookup(*id).name,
                storage::MetaTypeRecord::Interface(id) => self.lookup(*id).name,
                storage::MetaTypeRecord::Other(id) => self.lookup(*id).name,
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

    fn read(self, ast: &PartialCacheRegistry) -> Self::Reader<'_> {
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
    registry: &'a PartialCacheRegistry,
}

impl PartialCacheRegistry {
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
        field_types::MetaFieldTypeRecord,
        generated::{
            field::MetaFieldRecord, interface::InterfaceTypeRecord, metatype::MetaTypeRecord,
            objects::ObjectTypeRecord, others::OtherTypeRecord,
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
