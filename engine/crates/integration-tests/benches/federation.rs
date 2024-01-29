#![allow(unused_crate_dependencies)]

use criterion::{criterion_group, criterion_main, Criterion};
use integration_tests::federation::FederationGatewayBench;
use serde_json::json;

const SCHEMA: &str = r#"
directive @core(feature: String!) repeatable on SCHEMA

directive @join__owner(graph: join__Graph!) on OBJECT

directive @join__type(
    graph: join__Graph!
    key: String!
    resolvable: Boolean = true
) repeatable on OBJECT | INTERFACE

directive @join__field(
    graph: join__Graph
    requires: String
    provides: String
) on FIELD_DEFINITION

directive @join__graph(name: String!, url: String!) on ENUM_VALUE

enum join__Graph {
    ACCOUNTS @join__graph(name: "accounts", url: "http://127.0.0.1:46697")
    PRODUCTS @join__graph(name: "products", url: "http://127.0.0.1:45399")
    REVIEWS @join__graph(name: "reviews", url: "http://127.0.0.1:45899")
}

type Cart {
    products: [Product!]! @join__field(graph: ACCOUNTS)
}

type Picture {
    url: String!
    width: Int!
    height: Int!
    altText: String! @inaccessible
}

type Product
    @join__type(graph: ACCOUNTS, key: "name", resolvable: false)
    @join__type(graph: PRODUCTS, key: "upc")
    @join__type(graph: PRODUCTS, key: "name")
    @join__type(graph: REVIEWS, key: "upc")
{
    name: String!
    upc: String!
    price: Int! @join__field(graph: PRODUCTS)
    reviews: [Review!]! @join__field(graph: REVIEWS)
}

type User
    @join__type(graph: ACCOUNTS, key: "id")
    @join__type(graph: REVIEWS, key: "id")
{
    id: ID!
    username: String! @join__field(graph: ACCOUNTS)
    profilePicture: Picture @join__field(graph: ACCOUNTS)
    """
    This used to be part of this subgraph, but is now being overridden from
    `reviews`
    """
    reviewCount: Int! @join__field(graph: reviews, overrides: "accounts")
    joinedTimestamp: Int! @join__field(graph: ACCOUNTS)
    cart: Cart! @join__field(graph: ACCOUNTS)
    reviews: [Review!]! @join__field(graph: REVIEWS)
    trustworthiness: Trustworthiness! @join__field(graph: REVIEWS, requires: "joinedTimestamp")
}

type Review {
    id: ID! @join__field(graph: REVIEWS)
    body: String! @join__field(graph: REVIEWS)
    pictures: [Picture!]! @join__field(graph: REVIEWS)
    product: Product! @join__field(graph: REVIEWS, provides: "price")
    author: User @join__field(graph: REVIEWS)
}

type Query {
    me: User! @join__field(graph: ACCOUNTS)
    topProducts: [Product!]! @join__field(graph: PRODUCTS)
}

type Subscription {
    newProducts: Product! @join__field(graph: PRODUCTS)
}

enum Trustworthiness {
    REALLY_TRUSTED
    KINDA_TRUSTED
    NOT_TRUSTED
}
"#;

const PATHFINDER_INTROSPECTION_QUERY: &str = include_str!("../data/introspection.graphql");

pub fn introspection(c: &mut Criterion) {
    let bench = FederationGatewayBench::new(SCHEMA, PATHFINDER_INTROSPECTION_QUERY, &[json!({"data": null})]);
    let response = integration_tests::runtime().block_on(bench.execute());

    // Sanity check it works.
    insta::assert_snapshot!(String::from_utf8_lossy(&response.bytes));

    c.bench_function("introspection", |b| {
        // Insert a call to `to_async` to convert the bencher to async mode.
        // The timing loops are the same as with the normal bencher.
        b.to_async(
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap(),
        )
        .iter(|| bench.execute());
    });
}

pub fn basic_federation(c: &mut Criterion) {
    let bench = FederationGatewayBench::new(
        SCHEMA,
        r#"
        query ExampleQuery {
            me {
                id
                username
                reviews {
                    body
                    product {
                        reviews {
                            author {
                                id
                                username
                            }
                            body
                        }
                    }
                }
            }
        }
        "#,
        &[
            json!({"data":{"me":{"id":"1234","username":"Me"}}}),
            json!({"data":{"_entities":[{"__typename":"User","reviews":[{"body":"A highly effective form of birth control.","product":{"reviews":[{"author":{"id":"1234"},"body":"A highly effective form of birth control."}]}},{"body":"Fedoras are one of the most fashionable hats around and can look great with a variety of outfits.","product":{"reviews":[{"author":{"id":"1234"},"body":"Fedoras are one of the most fashionable hats around and can look great with a variety of outfits."}]}}]}]}}),
            json!({"data":{"_entities":[{"__typename":"User","username":"Me"},{"__typename":"User","username":"Me"}]}}),
        ],
    );
    let response = integration_tests::runtime().block_on(bench.execute());

    // Sanity check it works.
    insta::assert_snapshot!(String::from_utf8_lossy(&response.bytes));

    c.bench_function("basic_federation", |b| {
        // Insert a call to `to_async` to convert the bencher to async mode.
        // The timing loops are the same as with the normal bencher.
        b.to_async(
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap(),
        )
        .iter(|| bench.execute());
    });
}

criterion_group!(benches, introspection, basic_federation);
criterion_main!(benches);
