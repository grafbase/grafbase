use integration_tests::udfs::RustUdfs;
use integration_tests::{runtime, EngineBuilder};
use runtime::cache::Cacheable;
use runtime::udf::UdfResponse;
use serde_json::json;
use std::time::Duration;

#[test]
fn should_cache_with_entity_mutation_invalidation_custom_field() {
    let schema = r#"
        extend type Query {
            test: Post! @resolver(name: "test")
        }

        extend schema @cache(rules: [
                { maxAge: 60, staleWhileRevalidate: 10, types: [{name: "Post", fields: ["seconds"]}],  mutationInvalidation: { field: "seconds" } },
            ]
        )

        type Post {
            seconds: String
            hello: String
        }
    "#;

    runtime().block_on(async {
        let engine = EngineBuilder::new(schema)
            .with_custom_resolvers(RustUdfs::new().resolver(
                "test",
                UdfResponse::Success(json!({
                    "seconds": "test"
                })),
            ))
            .build()
            .await;

        // act
        let response = engine.execute("{ test { seconds } }").await;

        // assert
        let metadata = response.metadata();

        assert!(!metadata.should_purge_related);
        assert_eq!(metadata.tags, vec!["Post#seconds:test"]);
        assert_eq!(metadata.max_age, Duration::from_secs(60));
        assert_eq!(metadata.stale_while_revalidate, Duration::from_secs(10));
    });
}

#[test]
fn should_cache_with_entity_mutation_invalidation() {
    let schema = r#"
        extend type Query {
            test: Post! @resolver(name: "test")
        }

        extend schema @cache(rules: [
                { maxAge: 60, staleWhileRevalidate: 10, types: "Post",  mutationInvalidation: entity },
            ]
        )

        type Post {
            seconds: String
            hello: String
            id: ID!
        }
    "#;

    runtime().block_on(async {
        let engine = EngineBuilder::new(schema)
            .with_custom_resolvers(RustUdfs::new().resolver(
                "test",
                UdfResponse::Success(json!({
                    "seconds": "test",
                    "id": "hello",
                })),
            ))
            .build()
            .await;

        // act
        let response = engine.execute("{ test { id seconds } }").await;

        // assert
        let metadata = response.metadata();

        assert!(!metadata.should_purge_related);
        assert_eq!(metadata.tags, vec!["Post#id:hello"]);
        assert_eq!(metadata.max_age, Duration::from_secs(60));
        assert_eq!(metadata.stale_while_revalidate, Duration::from_secs(10));
    });
}

#[test]
fn should_cache_with_type_mutation_invalidation() {
    let schema = r#"
        extend type Query {
            test: Post! @resolver(name: "test")
        }

        extend schema @cache(rules: [
                { maxAge: 60, staleWhileRevalidate: 10, types: [{name: "Post", fields: ["seconds"]}],  mutationInvalidation: type },
            ]
        )

        type Post {
            seconds: String
            hello: String
        }
    "#;

    runtime().block_on(async {
        let engine = EngineBuilder::new(schema)
            .with_custom_resolvers(RustUdfs::new().resolver(
                "test",
                UdfResponse::Success(json!({
                    "seconds": "test"
                })),
            ))
            .build()
            .await;

        // act
        let response = engine.execute("{ test { seconds } }").await;

        // assert
        let metadata = response.metadata();

        assert!(!metadata.should_purge_related);
        assert_eq!(metadata.tags, vec!["Post"]);
        assert_eq!(metadata.max_age, Duration::from_secs(60));
        assert_eq!(metadata.stale_while_revalidate, Duration::from_secs(10));
    });
}

#[test]
fn should_cache_with_list_mutation_invalidation() {
    let schema = r#"
        extend type Query {
            test: Post! @resolver(name: "test")
        }

        extend schema @cache(rules: [
                { maxAge: 60, staleWhileRevalidate: 10, types: [{name: "Post", fields: ["seconds"]}],  mutationInvalidation: list },
            ]
        )

        type Post {
            seconds: String
            hello: String
        }
    "#;

    runtime().block_on(async {
        let engine = EngineBuilder::new(schema)
            .with_custom_resolvers(RustUdfs::new().resolver(
                "test",
                UdfResponse::Success(json!({
                    "seconds": "test"
                })),
            ))
            .build()
            .await;

        // act
        let response = engine.execute("{ test { seconds } }").await;

        // assert
        let metadata = response.metadata();

        assert!(!metadata.should_purge_related);
        assert_eq!(metadata.tags, vec!["Post#List"]);
        assert_eq!(metadata.max_age, Duration::from_secs(60));
        assert_eq!(metadata.stale_while_revalidate, Duration::from_secs(10));
    });
}

#[test]
fn should_not_cache_missing_field_from_response() {
    let schema = r#"
        extend type Query {
            test: Post! @resolver(name: "test")
        }

        extend schema @cache(rules: [
                { maxAge: 60, staleWhileRevalidate: 10, types: [{name: "Post", fields: ["seconds"]}] },
            ]
        )

        type Post {
            seconds: String
            hello: String
        }
    "#;

    runtime().block_on(async {
        let engine = EngineBuilder::new(schema)
            .with_custom_resolvers(RustUdfs::new().resolver(
                "test",
                UdfResponse::Success(json!({
                    "hello": "test"
                })),
            ))
            .build()
            .await;

        // act
        let response = engine.execute("{ test { hello } }").await;

        // assert
        let metadata = response.metadata();

        assert!(!metadata.should_purge_related);
        assert!(metadata.tags.is_empty());
        assert_eq!(metadata.max_age, Duration::from_secs(0));
        assert_eq!(metadata.stale_while_revalidate, Duration::from_secs(0));
    });
}

#[test]
fn should_purge_related_mutation_invalidation_entity() {
    let schema = r#"
        extend type Mutation {
            postCreate(name: String!): Post! @resolver(name: "test")
        }

        type Post @cache(maxAge: 10, staleWhileRevalidate: 10, mutationInvalidation: { field: "name" }) {
            id: ID!
            name: String!
        }
    "#;

    runtime().block_on(async {
        let engine = EngineBuilder::new(schema)
            .with_custom_resolvers(RustUdfs::new().resolver(
                "test",
                UdfResponse::Success(json!({
                    "id": "1",
                    "name": "hmm",
                })),
            ))
            .build()
            .await;

        // act
        let response = engine
            .execute("mutation { postCreate(name: \"hmm\") { id name } }")
            .await;

        // assert
        let metadata = response.metadata();

        assert!(metadata.should_purge_related);
        assert!(!metadata.should_cache);
        assert_eq!(metadata.tags, vec!["Post#name:hmm"]);
    });
}

#[test]
fn should_purge_related_mutation_invalidation_type() {
    let schema = r#"
        extend type Mutation {
            postCreate(name: String!): Post! @resolver(name: "test")
        }

        type Post @cache(maxAge: 10, staleWhileRevalidate: 10, mutationInvalidation: type) {
            id: ID!
            name: String!
        }
    "#;

    runtime().block_on(async {
        let engine = EngineBuilder::new(schema)
            .with_custom_resolvers(RustUdfs::new().resolver(
                "test",
                UdfResponse::Success(json!({
                    "id": "1",
                    "name": "hmm",
                })),
            ))
            .build()
            .await;

        // act
        let response = engine
            .execute("mutation { postCreate(name: \"hmm\") { id name } }")
            .await;

        // assert
        let metadata = response.metadata();

        assert!(metadata.should_purge_related);
        assert!(!metadata.should_cache);
        assert_eq!(metadata.tags, vec!["Post"]);
    });
}

#[test]
fn should_purge_related_mutation_invalidation_list() {
    let schema = r#"
        extend type Mutation {
            postCreate(name: String!): Post! @resolver(name: "test")
        }

        type Post @cache(maxAge: 10, staleWhileRevalidate: 10, mutationInvalidation: list) {
            id: ID!
            name: String!
        }
    "#;

    runtime().block_on(async {
        let engine = EngineBuilder::new(schema)
            .with_custom_resolvers(RustUdfs::new().resolver(
                "test",
                UdfResponse::Success(json!({
                    "id": "1",
                    "name": "hmm",
                })),
            ))
            .build()
            .await;

        // act
        let response = engine
            .execute("mutation { postCreate(name: \"hmm\") { id name } }")
            .await;

        // assert
        let metadata = response.metadata();

        assert!(metadata.should_purge_related);
        assert!(!metadata.should_cache);
        assert_eq!(metadata.tags, vec!["Post#List"]);
    });
}

#[test]
fn should_not_purge_related_mutation_invalidation_entity_missing_response_field() {
    let schema = r#"
        extend type Mutation {
            postCreate(name: String!): Post! @resolver(name: "test")
        }

        type Post @cache(maxAge: 10, staleWhileRevalidate: 10, mutationInvalidation: { field: "name" }) {
            id: ID!
            name: String
        }
    "#;

    runtime().block_on(async {
        let engine = EngineBuilder::new(schema)
            .with_custom_resolvers(RustUdfs::new().resolver(
                "test",
                UdfResponse::Success(json!({
                    "id": "1",
                })),
            ))
            .build()
            .await;

        // act
        let response = engine
            .execute("mutation { postCreate(name: \"hmm\") { id name } }")
            .await;

        // assert
        let metadata = response.metadata();

        assert!(!metadata.should_purge_related);
        assert!(!metadata.should_cache);
        assert!(metadata.tags.is_empty());
    });
}

#[test]
fn should_cache_fragments() {
    let schema = r#"
        extend type Query {
            test: Post! @resolver(name: "test")
        }

        extend schema @cache(rules: [
                { maxAge: 60, staleWhileRevalidate: 10, types: "Post",  mutationInvalidation: { field: "hello" } },
            ]
        )

        type Post {
            seconds: String
            hello: String
            id: ID!
        }
    "#;

    runtime().block_on(async {
        let engine = EngineBuilder::new(schema)
            .with_custom_resolvers(RustUdfs::new().resolver(
                "test",
                UdfResponse::Success(json!({
                    "id": "hello",
                    "hello": "fragment",
                })),
            ))
            .build()
            .await;

        // act
        let response = engine
            .execute(
                r"
            query { test { ...PostFrag } }

            fragment PostFrag on Post {
              hello
            }
        ",
            )
            .await;

        // assert
        let metadata = response.metadata();

        assert!(!metadata.should_purge_related);
        assert_eq!(metadata.tags, vec!["Post#hello:fragment"]);
        assert_eq!(metadata.max_age, Duration::from_secs(60));
        assert_eq!(metadata.stale_while_revalidate, Duration::from_secs(10));
    });
}

#[test]
fn should_purge_related_mutation_invalidation_fragments() {
    let schema = r#"
        extend type Mutation {
            postCreate(name: String!): Post! @resolver(name: "test")
        }

        type Post @cache(maxAge: 10, staleWhileRevalidate: 10, mutationInvalidation: { field: "name" }) {
            id: ID!
            name: String!
        }
    "#;

    runtime().block_on(async {
        let engine = EngineBuilder::new(schema)
            .with_custom_resolvers(RustUdfs::new().resolver(
                "test",
                UdfResponse::Success(json!({
                    "id": "1",
                    "name": "hmm",
                })),
            ))
            .build()
            .await;

        // act
        let response = engine
            .execute(
                r#"
            mutation { postCreate(name: "hmm") { ...PostFrag } }

            fragment PostFrag on Post {
              name
            }
        "#,
            )
            .await;

        // assert
        let metadata = response.metadata();

        assert!(metadata.should_purge_related);
        assert!(!metadata.should_cache);
        assert_eq!(metadata.tags, vec!["Post#name:hmm"]);
    });
}

#[test]
fn should_cache_interfaces() {
    let schema = r#"
        extend type Query {
            test: Post! @resolver(name: "test")
        }

        extend schema @cache(rules: [
                { maxAge: 60, staleWhileRevalidate: 10, types: "MyInterface",  mutationInvalidation: { field: "hello" } },
            ]
        )

        type Post implements MyInterface {
            seconds: String
            hello: String
            id: ID!
        }

        interface MyInterface {
          hello: String
        }
    "#;

    runtime().block_on(async {
        let engine = EngineBuilder::new(schema)
            .with_custom_resolvers(RustUdfs::new().resolver(
                "test",
                UdfResponse::Success(json!({
                    "id": "hello",
                    "hello": "fragment",
                })),
            ))
            .build()
            .await;

        // act
        let response = engine.execute(r"query { test { ... on MyInterface { hello } } }").await;

        // assert
        let metadata = response.metadata();

        assert!(!metadata.should_purge_related);
        assert_eq!(metadata.max_age, Duration::from_secs(60));
        assert_eq!(metadata.stale_while_revalidate, Duration::from_secs(10));
        assert_eq!(metadata.tags, vec!["Post#hello:fragment"]);
    });
}

#[test]
fn should_purge_related_mutation_invalidation_interfaces() {
    let schema = r#"
        extend type Mutation {
            postCreate(name: String!): MyInterface! @resolver(name: "test")
        }

        type Post implements MyInterface {
            id: ID!
            name: String!
        }

        interface MyInterface @cache(maxAge: 10, staleWhileRevalidate: 10, mutationInvalidation: { field: "name" }) {
          name: String!
        }
    "#;

    runtime().block_on(async {
        let engine = EngineBuilder::new(schema)
            .with_custom_resolvers(RustUdfs::new().resolver(
                "test",
                UdfResponse::Success(json!({
                    "id": "1",
                    "name": "hmm",
                })),
            ))
            .build()
            .await;

        // act
        let response = engine.execute(r#"mutation { postCreate(name: "hmm") { name } }"#).await;

        // assert
        let metadata = response.metadata();

        assert!(metadata.should_purge_related);
        assert!(!metadata.should_cache);
        assert_eq!(metadata.tags, vec!["MyInterface#name:hmm"]);
    });
}
