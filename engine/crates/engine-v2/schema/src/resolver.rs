use crate::{sources::*, SubgraphId};

/// A resolver is assumed to be specific to an object or an interface.
/// So multiple fields within an interface/object can share a resolver (like federation `@key`)
/// but different objects/interfaces MUST NOT share resolvers. To be more precise, they MUST NOT
/// share resolver *ids*. Resolvers are grouped and planned together by their id.
/// The only exception to this rule are resolvers on Mutation, Query and Subscription root types
/// as they can't be mixed together in a single operation. So a resolver id can be used
/// on both Query and Mutation fields.
#[derive(Debug, PartialEq, Eq)]
pub enum Resolver {
    Introspection(introspection::Resolver),
    FederationRootField(federation::RootFieldResolver),
    FederationEntity(federation::EntityResolver),
}

/// Resolvers within the same group are considered to be compatible. During planning, when
/// determining whether a resolver can provide nested fields we do not cross resolver boundaries.
/// So for example with:
/// ```graphql
/// query {
///   product {
///     name
///   }
/// }
/// ```
/// If `name` has resolver it will not be provided by the `product` resolver, except when both are
/// in the same group. In that case we consider that the `product` resolver can provide `name`.
///
/// A resolver does not need to have a group. If it doesn't than no nested fields with a resolvers
/// will be providable (unless overridden by `@provides`).
#[derive(Debug, PartialEq, Eq)]
pub enum ResolverGroup {
    FederationSubgraph(SubgraphId),
}
