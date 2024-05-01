mod partial_caching_registry;
mod v2;

/// A trait for registries that provides enough functionalty for the validation visitors
pub trait AnyRegistry {
    type MetaType<'a>: AnyMetaType<'a, Field = Self::Field<'a>>
    where
        Self: 'a;

    type Field<'a>: AnyField<'a, MetaType = Self::MetaType<'a>, MetaInputValue = Self::MetaInputValue<'a>>
    where
        Self: 'a;

    type MetaInputValue<'a>: AnyInputValue<'a, MetaType = Self::MetaType<'a>>
    where
        Self: 'a;

    type MetaDirective<'a>: AnyDirective<'a, MetaInputValue = Self::MetaInputValue<'a>>
    where
        Self: 'a;

    fn query_type(&self) -> Self::MetaType<'_>;
    fn mutation_type(&self) -> Option<Self::MetaType<'_>>;
    fn subscription_type(&self) -> Option<Self::MetaType<'_>>;

    fn lookup_type(&self, name: &str) -> Option<Self::MetaType<'_>>;

    fn directives(&self) -> impl Iterator<Item = Self::MetaDirective<'_>>;
}

pub trait AnyMetaType<'a>: Copy {
    type Field: AnyField<'a, MetaType = Self>;

    fn name(&self) -> &'a str;
    fn description(&self) -> Option<&'a str>;
    fn field(&self, name: &str) -> Option<Self::Field>;
    fn cache_control(&self) -> Option<&'a registry_v2::CacheControl>;
    fn possible_types(&self) -> Option<impl Iterator<Item = Self>>;

    fn is_input_object(&self) -> bool;
    fn input_field(&self, name: &str) -> Option<<Self::Field as AnyField<'a>>::MetaInputValue>;
}

pub trait AnyField<'a>: Copy {
    type MetaType: AnyMetaType<'a, Field = Self>;
    type MetaInputValue: AnyInputValue<'a>;

    fn named_type(&self) -> Self::MetaType;
    fn cache_control(&self) -> Option<&'a registry_v2::CacheControl>;
    fn argument(&self, name: &str) -> Option<Self::MetaInputValue>;
}

pub trait AnyInputValue<'a>: Copy {
    type MetaType: AnyMetaType<'a>;

    fn type_string(&self) -> String;
    fn named_type(&self) -> Self::MetaType;

    fn validators(&self) -> impl Iterator<Item = &'a registry_v2::validators::DynValidator>;
}

pub trait AnyDirective<'a>: Copy {
    type MetaInputValue: AnyInputValue<'a>;

    fn name(&self) -> &'a str;
    fn argument(&self, name: &str) -> Option<Self::MetaInputValue>;
}
