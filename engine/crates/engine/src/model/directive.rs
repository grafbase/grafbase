use std::collections::HashSet;

use grafbase_tracing::otel::tracing_subscriber::registry;

use crate::{model::__InputValue, Enum, Object};

/// A Directive can be adjacent to many parts of the GraphQL language, a __DirectiveLocation describes one such possible adjacencies.
#[derive(Debug, Enum, Copy, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[graphql(internal, name = "__DirectiveLocation")]
#[allow(non_camel_case_types)]
pub enum __DirectiveLocation {
    /// Location adjacent to a query operation.
    QUERY,

    /// Location adjacent to a mutation operation.
    MUTATION,

    /// Location adjacent to a subscription operation.
    SUBSCRIPTION,

    /// Location adjacent to a field.
    FIELD,

    /// Location adjacent to a fragment definition.
    FRAGMENT_DEFINITION,

    /// Location adjacent to a fragment spread.
    FRAGMENT_SPREAD,

    /// Location adjacent to an inline fragment.
    INLINE_FRAGMENT,

    /// Location adjacent to a variable definition.
    VARIABLE_DEFINITION,

    /// Location adjacent to a schema definition.
    SCHEMA,

    /// Location adjacent to a scalar definition.
    SCALAR,

    /// Location adjacent to an object type definition.
    OBJECT,

    /// Location adjacent to a field definition.
    FIELD_DEFINITION,

    /// Location adjacent to an argument definition.
    ARGUMENT_DEFINITION,

    /// Location adjacent to an interface definition.
    INTERFACE,

    /// Location adjacent to a union definition.
    UNION,

    /// Location adjacent to an enum definition.
    ENUM,

    /// Location adjacent to an enum value definition.
    ENUM_VALUE,

    /// Location adjacent to an input object type definition.
    INPUT_OBJECT,

    /// Location adjacent to an input object field definition.
    INPUT_FIELD_DEFINITION,
}

impl From<registry_v2::DirectiveLocation> for __DirectiveLocation {
    fn from(value: registry_v2::DirectiveLocation) -> Self {
        match value {
            registry_v2::DirectiveLocation::Query => __DirectiveLocation::QUERY,
            registry_v2::DirectiveLocation::Mutation => __DirectiveLocation::MUTATION,
            registry_v2::DirectiveLocation::Subscription => __DirectiveLocation::SUBSCRIPTION,
            registry_v2::DirectiveLocation::Field => __DirectiveLocation::FIELD,
            registry_v2::DirectiveLocation::FragmentDefinition => __DirectiveLocation::FRAGMENT_DEFINITION,
            registry_v2::DirectiveLocation::FragmentSpread => __DirectiveLocation::FRAGMENT_SPREAD,
            registry_v2::DirectiveLocation::InlineFragment => __DirectiveLocation::INLINE_FRAGMENT,
            registry_v2::DirectiveLocation::Schema => __DirectiveLocation::SCHEMA,
            registry_v2::DirectiveLocation::Scalar => __DirectiveLocation::SCALAR,
            registry_v2::DirectiveLocation::Object => __DirectiveLocation::OBJECT,
            registry_v2::DirectiveLocation::FieldDefinition => __DirectiveLocation::FIELD_DEFINITION,
            registry_v2::DirectiveLocation::ArgumentDefinition => __DirectiveLocation::ARGUMENT_DEFINITION,
            registry_v2::DirectiveLocation::Interface => __DirectiveLocation::INTERFACE,
            registry_v2::DirectiveLocation::Union => __DirectiveLocation::UNION,
            registry_v2::DirectiveLocation::Enum => __DirectiveLocation::ENUM,
            registry_v2::DirectiveLocation::EnumValue => __DirectiveLocation::ENUM_VALUE,
            registry_v2::DirectiveLocation::InputObject => __DirectiveLocation::INPUT_OBJECT,
            registry_v2::DirectiveLocation::InputFieldDefinition => __DirectiveLocation::INPUT_FIELD_DEFINITION,
            registry_v2::DirectiveLocation::VariableDefinition => __DirectiveLocation::VARIABLE_DEFINITION,
        }
    }
}

impl From<__DirectiveLocation> for registry_v2::DirectiveLocation {
    fn from(value: __DirectiveLocation) -> Self {
        match value {
            __DirectiveLocation::QUERY => registry_v2::DirectiveLocation::Query,
            __DirectiveLocation::MUTATION => registry_v2::DirectiveLocation::Mutation,
            __DirectiveLocation::SUBSCRIPTION => registry_v2::DirectiveLocation::Subscription,
            __DirectiveLocation::FIELD => registry_v2::DirectiveLocation::Field,
            __DirectiveLocation::FRAGMENT_DEFINITION => registry_v2::DirectiveLocation::FragmentDefinition,
            __DirectiveLocation::FRAGMENT_SPREAD => registry_v2::DirectiveLocation::FragmentSpread,
            __DirectiveLocation::INLINE_FRAGMENT => registry_v2::DirectiveLocation::InlineFragment,
            __DirectiveLocation::SCHEMA => registry_v2::DirectiveLocation::Schema,
            __DirectiveLocation::SCALAR => registry_v2::DirectiveLocation::Scalar,
            __DirectiveLocation::OBJECT => registry_v2::DirectiveLocation::Object,
            __DirectiveLocation::FIELD_DEFINITION => registry_v2::DirectiveLocation::FieldDefinition,
            __DirectiveLocation::ARGUMENT_DEFINITION => registry_v2::DirectiveLocation::ArgumentDefinition,
            __DirectiveLocation::INTERFACE => registry_v2::DirectiveLocation::Interface,
            __DirectiveLocation::UNION => registry_v2::DirectiveLocation::Union,
            __DirectiveLocation::ENUM => registry_v2::DirectiveLocation::Enum,
            __DirectiveLocation::ENUM_VALUE => registry_v2::DirectiveLocation::EnumValue,
            __DirectiveLocation::INPUT_OBJECT => registry_v2::DirectiveLocation::InputObject,
            __DirectiveLocation::INPUT_FIELD_DEFINITION => registry_v2::DirectiveLocation::InputFieldDefinition,
            __DirectiveLocation::VARIABLE_DEFINITION => registry_v2::DirectiveLocation::VariableDefinition,
        }
    }
}

pub struct __Directive<'a> {
    pub registry: &'a registry_v2::Registry,
    pub directive: registry_v2::MetaDirective<'a>,
}

/// A Directive provides a way to describe alternate runtime execution and type validation behavior in a GraphQL document.
///
/// In some cases, you need to provide options to alter GraphQL's execution behavior in ways field arguments will not suffice, such as conditionally including or skipping a field. Directives provide this by describing additional information to the executor.
#[Object(internal, name = "__Directive")]
impl<'a> __Directive<'a> {
    #[inline]
    async fn name(&self) -> &str {
        self.directive.name()
    }

    #[inline]
    async fn description(&self) -> Option<&str> {
        self.directive.description()
    }

    #[inline]
    async fn locations(&self) -> Vec<__DirectiveLocation> {
        self.directive.locations().map(Into::into).collect()
    }

    async fn args(&self) -> Vec<__InputValue<'a>> {
        self.directive
            .args()
            .map(|input_value| __InputValue {
                registry: self.registry,
                input_value,
            })
            .collect()
    }

    #[inline]
    async fn is_repeatable(&self) -> bool {
        self.directive.is_repeatable()
    }
}
