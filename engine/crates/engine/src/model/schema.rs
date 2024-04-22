use std::collections::HashSet;

use crate::{
    model::{__Directive, __Type},
    Object,
};

pub struct __Schema<'a> {
    registry: &'a registry_v2::Registry,
}

impl<'a> __Schema<'a> {
    pub fn new(registry: &'a registry_v2::Registry) -> Self {
        Self { registry }
    }
}

/// A GraphQL Schema defines the capabilities of a GraphQL server. It exposes all available types and directives on the server, as well as the entry points for query, mutation, and subscription operations.
#[Object(internal, name = "__Schema")]
impl<'a> __Schema<'a> {
    async fn description(&self) -> Option<String> {
        None
    }

    /// A list of all types supported by this server.
    async fn types(&self) -> Vec<__Type<'a>> {
        let mut types: Vec<_> = self
            .registry
            .types()
            .map(|ty| (ty.name(), __Type::new_simple(self.registry, ty)))
            .collect();
        types.sort_by(|a, b| a.0.cmp(b.0));
        types.into_iter().map(|(_, ty)| ty).collect()
    }

    /// The type that query operations will be rooted at.
    #[inline]
    async fn query_type(&self) -> __Type<'a> {
        __Type::new_simple(
            self.registry,
            self.registry.root_type(registry_v2::OperationType::Query).unwrap(),
        )
    }

    /// If this server supports mutation, the type that mutation operations will be rooted at.
    #[inline]
    async fn mutation_type(&self) -> Option<__Type<'a>> {
        self.registry
            .root_type(registry_v2::OperationType::Mutation)
            .map(|ty| __Type::new_simple(self.registry, ty))
    }

    /// If this server support subscription, the type that subscription operations will be rooted at.
    #[inline]
    async fn subscription_type(&self) -> Option<__Type<'a>> {
        self.registry
            .root_type(registry_v2::OperationType::Subscription)
            .map(|ty| __Type::new_simple(self.registry, ty))
    }

    /// A list of all directives supported by this server.
    async fn directives(&self) -> Vec<__Directive<'a>> {
        let mut directives: Vec<_> = self
            .registry
            .directives()
            .map(|directive| __Directive {
                registry: self.registry,
                directive,
            })
            .collect();
        directives.sort_by(|a, b| a.directive.name().cmp(&b.directive.name()));
        directives
    }
}
