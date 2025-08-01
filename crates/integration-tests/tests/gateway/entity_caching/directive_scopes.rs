//! Tests that we handle `@authenticated` & `@requiresScopes` directives on parent fields/types
//! correctly when doing entity caching

use graphql_mocks::{FederatedInventorySchema, FederatedProductsSchema, FederatedReviewsSchema, SecureFederatedSchema};
use integration_tests::{
    gateway::Gateway,
    openid::{CoreClientExt, JWKS_URI, OryHydraOpenIDProvider},
    runtime,
};

#[test]
#[ignore] // FIXME: fix cache scopes
fn test_authenticated_field_does_not_share_cache_with_unauthenticated() {
    runtime().block_on(async move {
        let token = jwt_token("read").await;

        let engine = engine().await;

        const AUTHENTICATED_QUERY: &str = "{ authenticatedProducts { upc reviews { id body } } }";
        const UNAUTHENTICATED_QUERY: &str = "{ topProducts { upc reviews { id body } } }";

        engine
            .post(AUTHENTICATED_QUERY)
            .header("Authorization", format!("Bearer {token}"))
            .await
            .into_data();
        engine
            .post(UNAUTHENTICATED_QUERY)
            .header("Authorization", format!("Bearer {token}"))
            .await
            .into_data();
        engine
            .post(AUTHENTICATED_QUERY)
            .header("Authorization", format!("Bearer {token}"))
            .await
            .into_data();

        assert_eq!(
            engine.drain_graphql_requests_sent_to::<FederatedReviewsSchema>().len(),
            2
        );
    })
}

#[test]
#[ignore] // FIXME: fix cache scopes
fn test_authenticated_type_does_not_share_cache_with_unauthenticated() {
    runtime().block_on(async move {
        let token = jwt_token("read").await;

        let engine = engine().await;

        const AUTHENTICATED_QUERY: &str = "{ authenticated { products { upc reviews { id body } } } }";
        const UNAUTHENTICATED_QUERY: &str = "{ topProducts { upc reviews { id body } } }";

        engine
            .post(AUTHENTICATED_QUERY)
            .header("Authorization", format!("Bearer {token}"))
            .await
            .into_data();
        engine
            .post(UNAUTHENTICATED_QUERY)
            .header("Authorization", format!("Bearer {token}"))
            .await
            .into_data();
        engine
            .post(AUTHENTICATED_QUERY)
            .header("Authorization", format!("Bearer {token}"))
            .await
            .into_data();

        assert_eq!(
            engine.drain_graphql_requests_sent_to::<FederatedReviewsSchema>().len(),
            2
        );
    })
}

#[test]
#[ignore] // FIXME: fix cache scopes
fn test_requires_scope_on_field() {
    runtime().block_on(async move {
        let read_token = jwt_token("read").await;
        let write_token = jwt_token("write").await;

        let engine = engine().await;

        const QUERY: &str = "{ scopedProducts { upc reviews { id body } } }";

        engine
            .post(QUERY)
            .header("Authorization", format!("Bearer {read_token}"))
            .await
            .into_data();
        engine
            .post(QUERY)
            .header("Authorization", format!("Bearer {read_token}"))
            .await
            .into_data();

        // The two above share scopes
        assert_eq!(
            engine.drain_graphql_requests_sent_to::<FederatedReviewsSchema>().len(),
            1
        );

        engine
            .post(QUERY)
            .header("Authorization", format!("Bearer {write_token}"))
            .await
            .into_data();

        assert_eq!(
            engine.drain_graphql_requests_sent_to::<FederatedReviewsSchema>().len(),
            1
        );
    })
}

#[test]
#[ignore] // FIXME: fix cache scopes
fn test_requires_scope_on_type() {
    runtime().block_on(async move {
        let read_token = jwt_token("read").await;
        let write_token = jwt_token("write").await;

        let engine = engine().await;

        const QUERY: &str = "{ scoped { products { upc reviews { id body } } } }";

        engine
            .post(QUERY)
            .header("Authorization", format!("Bearer {read_token}"))
            .await
            .into_data();
        engine
            .post(QUERY)
            .header("Authorization", format!("Bearer {read_token}"))
            .await
            .into_data();

        // The two above share scopes
        assert_eq!(
            engine.drain_graphql_requests_sent_to::<FederatedReviewsSchema>().len(),
            1
        );

        engine
            .post(QUERY)
            .header("Authorization", format!("Bearer {write_token}"))
            .await
            .into_data();

        assert_eq!(
            engine.drain_graphql_requests_sent_to::<FederatedReviewsSchema>().len(),
            1
        );
    })
}

async fn jwt_token(scope: &str) -> String {
    OryHydraOpenIDProvider::default()
        .create_client()
        .await
        .get_access_token_with_client_credentials(&[("scope", scope)])
        .await
}

async fn engine() -> Gateway {
    Gateway::builder()
        .with_subgraph(FederatedProductsSchema::default())
        .with_subgraph(FederatedReviewsSchema::default())
        .with_subgraph(SecureFederatedSchema::default())
        .with_subgraph(FederatedInventorySchema::default())
        .with_toml_config(format!(
            r#"
                [entity_caching]
                enabled = true

                [[authentication.providers]]
                [authentication.providers.jwt]
                name = "my-authenticator"

                [authentication.providers.jwt.jwks]
                url = "{JWKS_URI}"
                "#,
        ))
        .build()
        .await
}
