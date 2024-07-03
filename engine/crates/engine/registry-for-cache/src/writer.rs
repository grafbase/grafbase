use std::collections::BTreeMap;

use anyhow::anyhow;
use indexmap::IndexSet;

use crate::{
    ids::*,
    storage::{self, *},
    IdRange, PartialCacheRegistry,
};

/// Writes to a registry.
///
/// Note that this is a very low level interface.  We'd want some sort of friendly builders
/// built on top of this if we were ever to use it directly in parsers
#[derive(Default)]
pub struct RegistryWriter {
    strings: IndexSet<Box<str>>,

    // Optional so we can preallocate
    types: Vec<Option<storage::MetaTypeRecord>>,

    objects: Vec<storage::ObjectTypeRecord>,
    object_fields: Vec<storage::MetaFieldRecord>,

    interfaces: Vec<storage::InterfaceTypeRecord>,

    others: Vec<storage::OtherTypeRecord>,

    pub query_type: Option<MetaTypeId>,
    pub mutation_type: Option<MetaTypeId>,
    pub subscription_type: Option<MetaTypeId>,

    typename_to_supertypes: BTreeMap<StringId, IdRange<SupertypeId>>,
    supertypes: Vec<StringId>,

    pub enable_caching: bool,
    pub enable_partial_caching: bool,
}

impl RegistryWriter {
    pub fn new() -> Self {
        RegistryWriter::default()
    }

    pub fn preallocate_type_ids(&mut self, capacity: usize) -> impl ExactSizeIterator<Item = MetaTypeId> {
        let starting_id = MetaTypeId::new(self.types.len());
        self.types.extend(std::iter::repeat_with(|| None).take(capacity));

        IdRange::new(starting_id, MetaTypeId::new(self.types.len())).iter()
    }

    pub fn populate_preallocated_type(&mut self, id: MetaTypeId, record: MetaTypeRecord) {
        let index = id.to_index();
        if self.types[index].is_some() {
            panic!("Tried to populate an already populated index");
        }
        self.types[index] = Some(record);
    }

    #[must_use]
    pub fn insert_object(&mut self, details: ObjectTypeRecord) -> MetaTypeRecord {
        let id = ObjectTypeId::new(self.objects.len());
        self.objects.push(details);
        MetaTypeRecord::Object(id)
    }

    #[must_use]
    pub fn insert_interface(&mut self, details: InterfaceTypeRecord) -> MetaTypeRecord {
        let id = InterfaceTypeId::new(self.interfaces.len());
        self.interfaces.push(details);
        MetaTypeRecord::Interface(id)
    }

    #[must_use]
    pub fn insert_fields(&mut self, mut fields: Vec<MetaFieldRecord>) -> IdRange<MetaFieldId> {
        let starting_id = MetaFieldId::new(self.object_fields.len());

        // Sort the fields so we can binary search later
        fields.sort_by_key(|val| &self.strings[val.name.to_index()]);

        self.object_fields.append(&mut fields);

        IdRange::new(starting_id, MetaFieldId::new(self.object_fields.len()))
    }

    #[must_use]
    pub fn insert_other(&mut self, details: OtherTypeRecord) -> MetaTypeRecord {
        let id = OtherTypeId::new(self.others.len());
        self.others.push(details);
        MetaTypeRecord::Other(id)
    }

    pub fn insert_supertypes_for_type(&mut self, typename: String, subtypes: IdRange<SupertypeId>) {
        let typename = self.intern_string(typename);
        self.typename_to_supertypes.insert(typename, subtypes);
    }

    #[must_use]
    pub fn insert_supertypes(&mut self, supertype_names: Vec<String>) -> IdRange<SupertypeId> {
        let starting_index = self.supertypes.len();

        self.supertypes.reserve(supertype_names.len());
        for target in supertype_names {
            let id = self.intern_string(target);
            self.supertypes.push(id);
        }
        let ending_index = self.supertypes.len();

        self.supertypes[starting_index..ending_index].sort();

        SupertypeId::new(starting_index);
        IdRange::new(SupertypeId::new(starting_index), SupertypeId::new(ending_index))
    }

    #[must_use]
    pub fn intern_str(&mut self, string: &str) -> StringId {
        let (id, _) = self.strings.insert_full(string.into());
        StringId::new(id)
    }

    #[must_use]
    pub fn intern_string(&mut self, string: String) -> StringId {
        let (id, _) = self.strings.insert_full(string.into());
        StringId::new(id)
    }

    pub fn finish(self) -> anyhow::Result<PartialCacheRegistry> {
        let RegistryWriter {
            strings,
            types,
            objects,
            object_fields,
            interfaces,
            others,
            query_type,
            mutation_type,
            subscription_type,
            typename_to_supertypes: subtype_types,
            supertypes: subtype_targets,
            enable_caching,
            enable_partial_caching,
        } = self;

        let types = types
            .into_iter()
            .map(|ty| ty.ok_or_else(|| anyhow!("All preallocated types must be allocated")))
            .collect::<Result<Vec<_>, _>>()?;

        let query_type = query_type.ok_or_else(|| anyhow!("Root query type was not defined"))?;

        Ok(PartialCacheRegistry {
            strings,
            types,
            objects,
            object_fields,
            interfaces,
            others,
            query_type,
            mutation_type,
            subscription_type,
            typename_to_supertypes: subtype_types,
            supertypes: subtype_targets,
            enable_caching,
            enable_partial_caching,
        })
    }

    #[allow(dead_code)]
    fn meta_type_name_by_id(&self, id: MetaTypeId) -> &str {
        self.meta_type_name(
            self.types[id.to_index()]
                .as_ref()
                .expect("to be preopulated before this call"),
        )
    }

    fn meta_type_name(&self, record: &MetaTypeRecord) -> &str {
        let string_id = match record {
            MetaTypeRecord::Object(inner) => self.objects[inner.to_index()].name,
            MetaTypeRecord::Interface(inner) => self.interfaces[inner.to_index()].name,
            MetaTypeRecord::Other(inner) => self.others[inner.to_index()].name,
        };

        &self.strings[string_id.to_index()]
    }
}
