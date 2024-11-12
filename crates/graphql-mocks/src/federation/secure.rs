//! A mock GraphQL server that exposes authenticated & requiresScope directives
//! for testing their interaction with cache (and possibly other things)

use async_graphql::{EmptyMutation, EmptySubscription, Object, SimpleObject, TypeDirective};

pub struct SecureFederatedSchema;

impl crate::Subgraph for SecureFederatedSchema {
    fn name(&self) -> String {
        "secure".to_string()
    }

    async fn start(self) -> crate::MockGraphQlServer {
        crate::MockGraphQlServer::new(
            async_graphql::Schema::build(Query, EmptyMutation, EmptySubscription)
                .enable_federation()
                .finish(),
        )
        .await
    }
}

struct Query;

#[Object]
impl Query {
    #[graphql(
        directive = authenticated::apply()
    )]
    async fn authenticated_products(&self) -> Vec<Product> {
        vec![Product { upc: "top-1".into() }]
    }

    #[graphql(
        directive = requires_scopes::apply(vec![vec!["read".into()], vec!["write".into()]])
    )]
    async fn scoped_products(&self) -> Vec<Product> {
        vec![Product { upc: "top-1".into() }]
    }

    async fn authenticated(&self) -> AuthenticatedType {
        AuthenticatedType
    }

    async fn scoped(&self) -> ScopedType {
        ScopedType
    }
}

struct AuthenticatedType;

#[Object(
    directive = authenticated::apply()
)]
impl AuthenticatedType {
    async fn products(&self) -> Vec<Product> {
        vec![Product { upc: "top-1".into() }]
    }
}

struct ScopedType;

#[Object(
    directive = requires_scopes::apply(vec![vec!["read".into()], vec!["write".into()]])
)]
impl ScopedType {
    async fn products(&self) -> Vec<Product> {
        vec![Product { upc: "top-1".into() }]
    }
}

#[derive(Clone, SimpleObject)]
#[graphql(unresolvable = "upc")]
struct Product {
    upc: String,
}

#[TypeDirective(
    name = "federation__authenticated",
    location = "FieldDefinition",
    location = "Object",
    composable = "https://custom.spec.dev/extension/v1.0"
)]
fn authenticated() {}

#[TypeDirective(
    name = "federation__requiresScopes",
    location = "FieldDefinition",
    location = "Object",
    composable = "https://custom.spec.dev/extension/v1.0"
)]
fn requires_scopes(scopes: Vec<Vec<String>>) {}
