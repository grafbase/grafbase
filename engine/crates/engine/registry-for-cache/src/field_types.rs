mod serde_impls;

use crate::{ids::MetaTypeId, MetaField, MetaType, ReadContext, RecordLookup, RegistryId};

impl<'a> MetaField<'a> {
    pub fn ty(&self) -> MetaFieldType<'a> {
        let registry = &self.0.registry;
        registry.read(registry.lookup(self.0.id).ty)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MetaFieldTypeRecord {
    pub wrappers: wrapping::Wrapping,
    pub target: MetaTypeId,
}

pub struct MetaFieldType<'a>(pub(crate) ReadContext<'a, MetaFieldTypeRecord>);

impl<'a> MetaFieldType<'a> {
    pub fn id(&self) -> MetaFieldTypeRecord {
        self.0.id
    }

    pub fn is_list(&self) -> bool {
        self.0.id.wrappers.is_list()
    }

    pub fn is_non_null(&self) -> bool {
        self.0.id.wrappers.is_required()
    }

    pub fn is_nullable(&self) -> bool {
        self.0.id.wrappers.is_nullable()
    }

    pub fn typename(&self) -> &'a str {
        self.named_type().name()
    }

    pub fn named_type(&self) -> MetaType<'a> {
        let registry = self.0.registry;
        let MetaFieldTypeRecord { target, .. } = self.0.id;
        registry.read(target)
    }
}

impl std::fmt::Display for MetaFieldType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.id.wrappers.write_type_string(self.typename(), f)
    }
}

impl std::fmt::Debug for MetaFieldType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

impl RegistryId for MetaFieldTypeRecord {
    // This isn't technically an id but implementing this makes things work consistently
    type Reader<'a> = MetaFieldType<'a>;
}

impl<'a> From<ReadContext<'a, MetaFieldTypeRecord>> for MetaFieldType<'a> {
    fn from(value: ReadContext<'a, MetaFieldTypeRecord>) -> Self {
        Self(value)
    }
}
