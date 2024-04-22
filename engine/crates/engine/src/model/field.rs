use std::collections::HashSet;

use crate::{
    model::{__InputValue, __Type},
    ContextField, Object,
};

pub struct __Field<'a> {
    pub registry: &'a registry_v2::Registry,
    pub field: registry_v2::MetaField<'a>,
}

/// Object and Interface types are described by a list of Fields, each of which has a name, potentially a list of arguments, and a return type.
#[Object(internal, name = "__Field")]
impl<'a> __Field<'a> {
    #[inline]
    async fn name(&self) -> &str {
        self.field.name()
    }

    #[inline]
    async fn description(&self) -> Option<&str> {
        self.field.description()
    }

    async fn args(&self, _ctx: &ContextField<'_>) -> Vec<__InputValue<'a>> {
        self.field
            .args()
            // .filter(|input_value| is_visible(ctx, &input_value.visible))
            .map(|input_value| __InputValue {
                registry: self.registry,
                input_value,
            })
            .collect()
    }

    #[graphql(name = "type")]
    async fn ty(&self) -> __Type<'a> {
        __Type::new(self.registry, &self.field.ty().to_string())
    }

    #[inline]
    async fn is_deprecated(&self) -> bool {
        self.field
            .deprecation()
            .map(|depr| depr.is_deprecated())
            .unwrap_or_default()
    }

    #[inline]
    async fn deprecation_reason(&self) -> Option<&str> {
        self.field.deprecation().and_then(|depr| match &depr {
            registry_v2::Deprecation::NoDeprecated => None,
            registry_v2::Deprecation::Deprecated { reason } => Some(reason.as_ref()?.as_str()),
        })
    }
}
