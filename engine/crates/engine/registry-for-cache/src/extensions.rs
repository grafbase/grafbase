//! Extensions to the generated types

use crate::{ids::MetaFieldId, InterfaceType, Iter, MetaField, MetaType, ObjectType, RecordLookup};

impl<'a> MetaType<'a> {
    pub fn name(&self) -> &'a str {
        match self {
            MetaType::Object(object) => object.name(),
            MetaType::Interface(iface) => iface.name(),
            MetaType::Other(other) => other.name(),
        }
    }

    pub fn fields(&self) -> Option<Iter<'a, MetaField<'a>>> {
        match self {
            MetaType::Object(inner) => Some(inner.fields()),
            MetaType::Interface(inner) => Some(inner.fields()),
            MetaType::Other(_) => None,
        }
    }

    pub fn cache_control(&self) -> Option<&'a registry_v2::CacheControl> {
        match self {
            MetaType::Object(object) => object.cache_control(),
            MetaType::Interface(iface) => iface.cache_control(),
            MetaType::Other(_) => None,
        }
    }
}

impl<'a> MetaType<'a> {
    pub fn field(&self, name: &str) -> Option<MetaField<'a>> {
        match self {
            MetaType::Object(obj) => obj.field(name),
            MetaType::Interface(iface) => iface.field(name),
            MetaType::Other(_) => None,
        }
    }
}

impl<'a> ObjectType<'a> {
    pub fn field(&self, name: &str) -> Option<MetaField<'a>> {
        let object = self.0.registry.lookup(self.0.id);
        let index = self.0.registry.object_fields[object.fields.start.to_index()..object.fields.end.to_index()]
            .binary_search_by(|field| self.0.registry.string_cmp(field.name, name))
            .ok()?;

        Some(
            self.0
                .registry
                .read(MetaFieldId::new(object.fields.start.to_index() + index)),
        )
    }
}

impl<'a> InterfaceType<'a> {
    pub fn field(&self, name: &str) -> Option<MetaField<'a>> {
        let object = self.0.registry.lookup(self.0.id);
        let index = self.0.registry.object_fields[object.fields.start.to_index()..object.fields.end.to_index()]
            .binary_search_by(|field| self.0.registry.string_cmp(field.name, name))
            .ok()?;

        Some(
            self.0
                .registry
                .read(MetaFieldId::new(object.fields.start.to_index() + index)),
        )
    }
}
