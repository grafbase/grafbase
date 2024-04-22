mod serde_impls;

use crate::{
    ids::MetaTypeId, MetaField, MetaInputValue, MetaType, ReadContext, RecordLookup, RegistryId, WrappingType,
};

impl<'a> MetaField<'a> {
    pub fn ty(&self) -> MetaFieldType<'a> {
        let registry = &self.0.registry;
        registry.read(registry.lookup(self.0.id).ty)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MetaFieldTypeRecord {
    pub wrappers: crate::common::TypeWrappers,
    pub target: MetaTypeId,
}

pub struct MetaFieldType<'a>(pub(crate) ReadContext<'a, MetaFieldTypeRecord>);

impl<'a> MetaFieldType<'a> {
    pub fn id(&self) -> MetaFieldTypeRecord {
        self.0.id
    }

    pub fn is_list(&self) -> bool {
        self.0
            .id
            .wrappers
            .iter()
            .any(|wrapper| matches!(wrapper, WrappingType::List))
    }

    pub fn is_non_null(&self) -> bool {
        self.0
            .id
            .wrappers
            .iter()
            .next()
            .map(|wrapper| matches!(wrapper, WrappingType::NonNull))
            .unwrap_or_default()
    }

    pub fn is_nullable(&self) -> bool {
        !self.is_non_null()
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

impl<'a> MetaInputValue<'a> {
    pub fn ty(&self) -> MetaInputValueType<'a> {
        let registry = &self.0.registry;
        registry.read(registry.lookup(self.0.id).ty)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MetaInputValueTypeRecord {
    pub wrappers: crate::common::TypeWrappers,
    pub target: MetaTypeId,
}

pub struct MetaInputValueType<'a>(pub(crate) ReadContext<'a, MetaInputValueTypeRecord>);

impl<'a> MetaInputValueType<'a> {
    pub fn id(&self) -> MetaInputValueTypeRecord {
        self.0.id
    }

    pub fn is_list(&self) -> bool {
        self.0
            .id
            .wrappers
            .iter()
            .any(|wrapper| matches!(wrapper, WrappingType::List))
    }

    pub fn is_non_null(&self) -> bool {
        self.0
            .id
            .wrappers
            .iter()
            .next()
            .map(|wrapper| matches!(wrapper, WrappingType::NonNull))
            .unwrap_or_default()
    }

    pub fn is_nullable(&self) -> bool {
        !self.is_non_null()
    }

    pub fn typename(&self) -> &'a str {
        self.named_type().name()
    }

    pub fn named_type(&self) -> MetaType<'a> {
        let registry = self.0.registry;
        let MetaInputValueTypeRecord { target, .. } = self.0.id;
        registry.read(target)
    }
}

impl std::fmt::Display for MetaInputValueType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.id.wrappers.write_type_string(self.typename(), f)
    }
}

impl std::fmt::Debug for MetaInputValueType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

impl RegistryId for MetaInputValueTypeRecord {
    // This isn't technically an id but implementing this makes things work consistently
    type Reader<'a> = MetaInputValueType<'a>;
}

impl<'a> From<ReadContext<'a, MetaInputValueTypeRecord>> for MetaInputValueType<'a> {
    fn from(value: ReadContext<'a, MetaInputValueTypeRecord>) -> Self {
        Self(value)
    }
}
