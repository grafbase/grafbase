mod partial_caching_registry;
mod v2;

/// A trait for registries that provides enough functionalty for the validation visitors
pub(crate) trait ValidationRegistry {
    type MetaType<'a>: ValidationMetaType<'a>
    where
        Self: 'a;

    type MetaInputValue<'a>: Copy;

    fn query_type(&self) -> Self::MetaType<'_>;
    fn mutation_type(&self) -> Option<Self::MetaType<'_>>;
    fn subscription_type(&self) -> Option<Self::MetaType<'_>>;

    fn lookup_type(&self, name: &str) -> Option<Self::MetaType<'_>>;
}

pub(crate) trait ValidationMetaType<'a>: Copy {
    type Field: ValidationField<'a, MetaType = Self>;

    fn name(&self) -> &str;
    fn description(&self) -> Option<&str>;
    fn field(&self, name: &str) -> Option<Self::Field>;
    fn cache_control(&self) -> Option<registry_v2::CacheControl>;
    fn possible_types(&self) -> Option<impl Iterator<Item = Self::Field>>;
}

pub(crate) trait ValidationField<'a>: Copy {
    type MetaType: ValidationMetaType<'a, Field = Self>;

    fn named_type(&self) -> Self::MetaType;
    fn cache_control(&self) -> Option<registry_v2::CacheControl>;
}
