use std::time::Duration;

use engine_v2::Engine;
use graphql_mocks::{FederatedInventorySchema, FederatedProductsSchema, FederatedReviewsSchema};
use headers::{Age, CacheControl};
use integration_tests::{federation::EngineV2Ext, runtime};

struct CacheControlReviewSubgraph {
    header: CacheControl,
    age: Option<Age>,
}

impl graphql_mocks::Subgraph for CacheControlReviewSubgraph {
    fn name(&self) -> String {
        "reviews".into()
    }

    async fn start(self) -> graphql_mocks::MockGraphQlServer {
        let mut server = FederatedReviewsSchema.start().await.with_additional_header(self.header);

        if let Some(age) = self.age {
            server = server.with_additional_header(age);
        }

        server
    }
}

struct CacheControlProductSubgraph {
    header: CacheControl,
    age: Option<Age>,
}

impl graphql_mocks::Subgraph for CacheControlProductSubgraph {
    fn name(&self) -> String {
        "products".into()
    }

    async fn start(self) -> graphql_mocks::MockGraphQlServer {
        let mut server = FederatedProductsSchema
            .start()
            .await
            .with_additional_header(self.header);

        if let Some(age) = self.age {
            server = server.with_additional_header(age);
        }

        server
    }
}

#[test]
fn test_private_cache_control_entity_request() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(FederatedProductsSchema)
            .with_subgraph(CacheControlReviewSubgraph {
                header: CacheControl::new().with_private(),
                age: None,
            })
            .with_subgraph(FederatedInventorySchema)
            .with_toml_config(
                r#"
                [entity_caching]
                enabled = true
                "#,
            )
            .build()
            .await;

        const QUERY: &str = "{ topProducts { upc reviews { id body } } }";

        engine.post(QUERY).await.into_data();
        engine.post(QUERY).await.into_data();

        assert_eq!(
            engine.drain_graphql_requests_sent_to::<FederatedProductsSchema>().len(),
            1
        );
        assert_eq!(
            engine
                .drain_graphql_requests_sent_to::<CacheControlReviewSubgraph>()
                .len(),
            2
        );
    })
}

#[test]
fn test_nostore_cache_control_entity_request() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(FederatedProductsSchema)
            .with_subgraph(CacheControlReviewSubgraph {
                header: CacheControl::new().with_no_store(),
                age: None,
            })
            .with_subgraph(FederatedInventorySchema)
            .with_toml_config(
                r#"
                [entity_caching]
                enabled = true
                "#,
            )
            .build()
            .await;

        const QUERY: &str = "{ topProducts { upc reviews { id body } } }";

        engine.post(QUERY).await.into_data();
        engine.post(QUERY).await.into_data();

        assert_eq!(
            engine.drain_graphql_requests_sent_to::<FederatedProductsSchema>().len(),
            1
        );
        assert_eq!(
            engine
                .drain_graphql_requests_sent_to::<CacheControlReviewSubgraph>()
                .len(),
            2
        );
    })
}

#[test]
fn test_max_age_without_age_entity_request() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(FederatedProductsSchema)
            .with_subgraph(CacheControlReviewSubgraph {
                header: CacheControl::new().with_max_age(Duration::from_secs(1)),
                age: None,
            })
            .with_subgraph(FederatedInventorySchema)
            .with_toml_config(
                r#"
                [entity_caching]
                enabled = true
                "#,
            )
            .build()
            .await;

        const QUERY: &str = "{ topProducts { upc reviews { id body } } }";

        engine.post(QUERY).await.into_data();
        engine.post(QUERY).await.into_data();

        tokio::time::sleep(Duration::from_millis(1200)).await;

        engine.post(QUERY).await.into_data();

        assert_eq!(
            engine.drain_graphql_requests_sent_to::<FederatedProductsSchema>().len(),
            1
        );
        assert_eq!(
            engine
                .drain_graphql_requests_sent_to::<CacheControlReviewSubgraph>()
                .len(),
            2
        );
    })
}

#[test]
fn test_max_age_with_age_entity_request() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(FederatedProductsSchema)
            .with_subgraph(CacheControlReviewSubgraph {
                header: CacheControl::new().with_max_age(Duration::from_secs(2)),
                age: Some(Age::from_secs(1)),
            })
            .with_subgraph(FederatedInventorySchema)
            .with_toml_config(
                r#"
                [entity_caching]
                enabled = true
                "#,
            )
            .build()
            .await;

        const QUERY: &str = "{ topProducts { upc reviews { id body } } }";

        engine.post(QUERY).await.into_data();
        engine.post(QUERY).await.into_data();

        tokio::time::sleep(Duration::from_millis(1200)).await;

        engine.post(QUERY).await.into_data();

        assert_eq!(
            engine.drain_graphql_requests_sent_to::<FederatedProductsSchema>().len(),
            1
        );
        assert_eq!(
            engine
                .drain_graphql_requests_sent_to::<CacheControlReviewSubgraph>()
                .len(),
            2
        );
    })
}

#[test]
fn test_private_cache_control_root_request() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(CacheControlProductSubgraph {
                header: CacheControl::new().with_private(),
                age: None,
            })
            .with_subgraph(FederatedReviewsSchema)
            .with_subgraph(FederatedInventorySchema)
            .with_toml_config(
                r#"
                [entity_caching]
                enabled = true
                "#,
            )
            .build()
            .await;

        const QUERY: &str = "{ topProducts { upc reviews { id body } } }";

        engine.post(QUERY).await.into_data();
        engine.post(QUERY).await.into_data();

        assert_eq!(
            engine
                .drain_graphql_requests_sent_to::<CacheControlProductSubgraph>()
                .len(),
            2
        );
        assert_eq!(
            engine.drain_graphql_requests_sent_to::<FederatedReviewsSchema>().len(),
            1
        );
    })
}

#[test]
fn test_nostore_cache_control_root_request() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(CacheControlProductSubgraph {
                header: CacheControl::new().with_no_store(),
                age: None,
            })
            .with_subgraph(FederatedReviewsSchema)
            .with_subgraph(FederatedInventorySchema)
            .with_toml_config(
                r#"
                [entity_caching]
                enabled = true
                "#,
            )
            .build()
            .await;

        const QUERY: &str = "{ topProducts { upc reviews { id body } } }";

        engine.post(QUERY).await.into_data();
        engine.post(QUERY).await.into_data();

        assert_eq!(
            engine
                .drain_graphql_requests_sent_to::<CacheControlProductSubgraph>()
                .len(),
            2
        );
        assert_eq!(
            engine.drain_graphql_requests_sent_to::<FederatedReviewsSchema>().len(),
            1
        );
    })
}

#[test]
fn test_max_age_without_age_root_request() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(CacheControlProductSubgraph {
                header: CacheControl::new().with_max_age(Duration::from_secs(1)),
                age: None,
            })
            .with_subgraph(FederatedReviewsSchema)
            .with_subgraph(FederatedInventorySchema)
            .with_toml_config(
                r#"
                [entity_caching]
                enabled = true
                "#,
            )
            .build()
            .await;

        const QUERY: &str = "{ topProducts { upc reviews { id body } } }";

        engine.post(QUERY).await.into_data();
        engine.post(QUERY).await.into_data();

        tokio::time::sleep(Duration::from_millis(1200)).await;

        engine.post(QUERY).await.into_data();

        assert_eq!(
            engine
                .drain_graphql_requests_sent_to::<CacheControlProductSubgraph>()
                .len(),
            2
        );
        assert_eq!(
            engine.drain_graphql_requests_sent_to::<FederatedReviewsSchema>().len(),
            1
        );
    })
}

#[test]
fn test_max_age_with_age_root_request() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(CacheControlProductSubgraph {
                header: CacheControl::new().with_max_age(Duration::from_secs(2)),
                age: Some(Age::from_secs(1)),
            })
            .with_subgraph(FederatedReviewsSchema)
            .with_subgraph(FederatedInventorySchema)
            .with_toml_config(
                r#"
                [entity_caching]
                enabled = true
                "#,
            )
            .build()
            .await;

        const QUERY: &str = "{ topProducts { upc reviews { id body } } }";

        engine.post(QUERY).await.into_data();
        engine.post(QUERY).await.into_data();

        tokio::time::sleep(Duration::from_millis(1200)).await;

        engine.post(QUERY).await.into_data();

        assert_eq!(
            engine
                .drain_graphql_requests_sent_to::<CacheControlProductSubgraph>()
                .len(),
            2
        );
        assert_eq!(
            engine.drain_graphql_requests_sent_to::<FederatedReviewsSchema>().len(),
            1
        );
    })
}
