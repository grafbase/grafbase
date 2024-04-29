use registry_v2::{MetaType, Registry};

use super::{ValidationMetaType, ValidationRegistry};

impl ValidationRegistry for Registry {
    type MetaType<'a> = MetaType<'a>;

    type MetaInputValue<'a> = registry_v2::MetaInputValue<'a>;

    fn query_type(&self) -> Self::MetaType<'_> {
        Registry::query_type(&self)
    }

    fn mutation_type(&self) -> Option<Self::MetaType<'_>> {
        Registry::mutation_type(&self)
    }

    fn subscription_type(&self) -> Option<Self::MetaType<'_>> {
        Registry::subscription_type(&self)
    }

    fn lookup_type(&self, name: &str) -> Option<Self::MetaType<'_>> {
        todo!()
    }
}

impl<'a> ValidationMetaType<'a> for MetaType<'a> {
    type Field = ();

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
