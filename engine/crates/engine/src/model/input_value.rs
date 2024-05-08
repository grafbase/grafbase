use crate::{model::__Type, Object};

pub struct __InputValue<'a> {
    pub registry: &'a registry_v2::Registry,
    pub input_value: registry_v2::MetaInputValue<'a>,
}

/// Arguments provided to Fields or Directives and the input fields of an InputObject are represented as Input Values which describe their type and optionally a default value.
#[Object(internal, name = "__InputValue")]
impl<'a> __InputValue<'a> {
    #[inline]
    async fn name(&self) -> &str {
        self.input_value.name()
    }

    #[inline]
    async fn description(&self) -> Option<&str> {
        self.input_value.description()
    }

    #[graphql(name = "type")]
    #[inline]
    async fn ty(&self) -> __Type<'a> {
        __Type::new(self.registry, &self.input_value.ty().to_string())
    }

    #[inline]
    async fn default_value(&self) -> Option<String> {
        self.input_value.default_value().map(std::string::ToString::to_string)
    }
}
