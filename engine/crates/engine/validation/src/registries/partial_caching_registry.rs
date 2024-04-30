use registry_for_cache::{MetaField, MetaType, PartialCacheRegistry};

use super::{AnyDirective, AnyField, AnyInputValue, AnyMetaType, AnyRegistry};

impl AnyRegistry for PartialCacheRegistry {
    type MetaType<'a> = MetaType<'a>;
    type Field<'a> = MetaField<'a>;
    type MetaDirective<'a> = Never;

    type MetaInputValue<'a> = Never;

    fn query_type(&self) -> Self::MetaType<'_> {
        PartialCacheRegistry::query_type(self)
    }

    fn mutation_type(&self) -> Option<Self::MetaType<'_>> {
        PartialCacheRegistry::mutation_type(self)
    }

    fn subscription_type(&self) -> Option<Self::MetaType<'_>> {
        PartialCacheRegistry::subscription_type(self)
    }

    fn lookup_type(&self, name: &str) -> Option<Self::MetaType<'_>> {
        PartialCacheRegistry::lookup_type(self, name)
    }

    fn directives(&self) -> impl Iterator<Item = Self::MetaDirective<'_>> {
        [].into_iter()
    }
}

impl<'a> AnyMetaType<'a> for MetaType<'a> {
    type Field = MetaField<'a>;

    fn name(&self) -> &'a str {
        MetaType::name(self)
    }

    fn description(&self) -> Option<&'a str> {
        // We don't do descriptions in the caching registry
        None
    }

    fn field(&self, name: &str) -> Option<Self::Field> {
        MetaType::field(self, name)
    }

    fn cache_control(&self) -> Option<&'a registry_v2::CacheControl> {
        MetaType::cache_control(self)
    }

    fn possible_types(&self) -> Option<impl Iterator<Item = Self>> {
        match self {
            MetaType::Object(_) => None,
            MetaType::Interface(interface) => Some(interface.possible_types()),
        }
    }

    fn is_input_object(&self) -> bool {
        // Caching registry has no input objects
        false
    }

    fn input_field(&self, _name: &str) -> Option<<Self::Field as AnyField<'a>>::MetaInputValue> {
        // Caching registry has no input fields
        None
    }
}

impl<'a> AnyField<'a> for MetaField<'a> {
    type MetaType = MetaType<'a>;
    type MetaInputValue = Never;

    fn named_type(&self) -> Self::MetaType {
        MetaField::ty(self).named_type()
    }

    fn cache_control(&self) -> Option<&'a registry_v2::CacheControl> {
        MetaField::cache_control(self)
    }

    fn argument(&self, _name: &str) -> Option<Self::MetaInputValue> {
        // Caching registry has no arguments
        None
    }
}

#[derive(Clone, Copy)]
pub enum Never {}

impl<'a> AnyInputValue<'a> for Never {
    type MetaType = MetaType<'a>;

    fn type_string(&self) -> String {
        unimplemented!()
    }

    fn named_type(&self) -> Self::MetaType {
        unimplemented!()
    }

    #[allow(unreachable_code)]
    fn validators(&self) -> impl Iterator<Item = &'a registry_v2::validators::DynValidator> {
        unimplemented!();

        // We need this here so the impl knows what type we're returning
        [].into_iter()
    }
}

impl<'a> AnyDirective<'a> for Never {
    type MetaInputValue = Never;

    fn argument(&self, _name: &str) -> Option<Self::MetaInputValue> {
        unimplemented!()
    }

    fn name(&self) -> &'a str {
        unimplemented!()
    }
}
