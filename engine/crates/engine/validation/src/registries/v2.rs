use registry_v2::*;

use super::{AnyDirective, AnyField, AnyInputValue, AnyMetaType, AnyRegistry};

impl AnyRegistry for Registry {
    type MetaType<'a> = MetaType<'a>;
    type Field<'a> = MetaField<'a>;
    type MetaDirective<'a> = MetaDirective<'a>;

    type MetaInputValue<'a> = registry_v2::MetaInputValue<'a>;

    fn query_type(&self) -> Self::MetaType<'_> {
        Registry::query_type(self)
    }

    fn mutation_type(&self) -> Option<Self::MetaType<'_>> {
        Registry::mutation_type(self)
    }

    fn subscription_type(&self) -> Option<Self::MetaType<'_>> {
        Registry::subscription_type(self)
    }

    fn lookup_type(&self, name: &str) -> Option<Self::MetaType<'_>> {
        Registry::lookup_type(self, name)
    }

    fn directives(&self) -> impl Iterator<Item = Self::MetaDirective<'_>> {
        Registry::directives(self)
    }
}

impl<'a> AnyMetaType<'a> for MetaType<'a> {
    type Field = MetaField<'a>;

    fn name(&self) -> &'a str {
        MetaType::name(self)
    }

    fn description(&self) -> Option<&'a str> {
        MetaType::description(self)
    }

    fn field(&self, name: &str) -> Option<Self::Field> {
        MetaType::field(self, name)
    }

    fn cache_control(&self) -> Option<&'a registry_v2::CacheControl> {
        MetaType::cache_control(self)
    }

    fn possible_types(&self) -> Option<impl Iterator<Item = Self>> {
        MetaType::possible_types(self)
    }

    fn is_input_object(&self) -> bool {
        matches!(self, MetaType::InputObject(_))
    }

    fn input_field(&self, name: &str) -> Option<<Self::Field as AnyField<'a>>::MetaInputValue> {
        match self {
            MetaType::InputObject(obj) => obj.field(name),
            _ => None,
        }
    }
}

impl<'a> AnyField<'a> for MetaField<'a> {
    type MetaType = MetaType<'a>;
    type MetaInputValue = MetaInputValue<'a>;

    fn named_type(&self) -> Self::MetaType {
        MetaField::ty(self).named_type()
    }

    fn cache_control(&self) -> Option<&'a registry_v2::CacheControl> {
        MetaField::cache_control(self)
    }

    fn argument(&self, name: &str) -> Option<Self::MetaInputValue> {
        MetaField::argument(self, name)
    }
}

impl<'a> AnyInputValue<'a> for MetaInputValue<'a> {
    type MetaType = MetaType<'a>;

    fn type_string(&self) -> String {
        MetaInputValue::ty(self).to_string()
    }

    fn named_type(&self) -> Self::MetaType {
        MetaInputValue::ty(self).named_type()
    }

    fn validators(&self) -> impl Iterator<Item = &'a registry_v2::validators::DynValidator> {
        MetaInputValue::validators(self).map(|v| v.validator())
    }
}

impl<'a> AnyDirective<'a> for MetaDirective<'a> {
    type MetaInputValue = MetaInputValue<'a>;

    fn argument(&self, name: &str) -> Option<Self::MetaInputValue> {
        MetaDirective::argument(self, name)
    }

    fn name(&self) -> &'a str {
        MetaDirective::name(self)
    }
}
