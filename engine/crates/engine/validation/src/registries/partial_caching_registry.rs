use registry_for_cache::{MetaField, MetaType, PartialCacheRegistry};

use super::{ValidationMetaType, ValidationRegistry};

impl ValidationRegistry for PartialCacheRegistry {
    type MetaType<'a> = MetaType<'a>;

    type MetaInputValue<'a> = Never;

    fn query_type(&self) -> Self::MetaType<'_> {
        PartialCacheRegistry::query_type(&self)
    }

    fn mutation_type(&self) -> Option<Self::MetaType<'_>> {
        PartialCacheRegistry::mutation_type(&self)
    }

    fn subscription_type(&self) -> Option<Self::MetaType<'_>> {
        PartialCacheRegistry::subscription_type(&self)
    }

    fn lookup_type(&self, name: &str) -> Option<Self::MetaType<'_>> {
        todo!()
    }
}

impl<'a> ValidationMetaType<'a> for MetaType<'a> {
    type Field = MetaField<'a>;

    fn name(&self) -> &str {
        todo!()
    }

    fn description(&self) -> Option<&str> {
        todo!()
    }

    fn field(&self, name: &str) -> Option<Self::Field> {
        todo!()
    }

    fn cache_control(&self) -> Option<registry_v2::CacheControl> {
        todo!()
    }

    fn possible_types(&self) -> Option<impl Iterator<Item = Self::Field>> {
        todo!()
    }
}

#[derive(Clone, Copy)]
enum Never {}
