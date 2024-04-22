use crate::Object;

pub struct __EnumValue<'a> {
    pub registry: &'a registry_v2::Registry,
    pub value: registry_v2::MetaEnumValue<'a>,
}

/// One possible value for a given Enum. Enum values are unique values, not a placeholder for a string or numeric value. However an Enum value is returned in a JSON response as a string.
#[Object(internal, name = "__EnumValue")]
impl<'a> __EnumValue<'a> {
    #[inline]
    async fn name(&self) -> &str {
        self.value.name()
    }

    #[inline]
    async fn description(&self) -> Option<&str> {
        self.value.description()
    }

    #[inline]
    async fn is_deprecated(&self) -> bool {
        self.value
            .deprecation()
            .map(|depr| depr.is_deprecated())
            .unwrap_or_default()
    }

    #[inline]
    async fn deprecation_reason(&self) -> Option<&str> {
        self.value.deprecation().and_then(|depr| match &depr {
            registry_v2::Deprecation::NoDeprecated => None,
            registry_v2::Deprecation::Deprecated { reason } => Some(reason.as_ref()?.as_str()),
        })
    }
}
