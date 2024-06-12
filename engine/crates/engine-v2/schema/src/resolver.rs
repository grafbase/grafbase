use crate::sources::*;

/// A resolver is assumed to be specific to an object or an interface.
/// So multiple fields within an interface/object can share a resolver (like federation `@key`)
/// but different objects/interfaces MUST NOT share resolvers. To be more precise, they MUST NOT
/// share resolver *ids*. Resolvers are grouped and planned together by their id.
/// The only exception to this rule are resolvers on Mutation, Query and Subscription root types
/// as they can't be mixed together in a single operation. So a resolver id can be used
/// on both Query and Mutation fields.
#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Resolver {
    Introspection(introspection::Resolver),
    GraphqlRootField(graphql::RootFieldResolver),
    GraphqlFederationEntity(graphql::FederationEntityResolver),
}
