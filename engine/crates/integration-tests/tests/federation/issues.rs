use engine_v2::Engine;
use integration_tests::{federation::EngineV2Ext, fetch::MockFetch, runtime};
use serde_json::json;

#[test]
fn gb6873_wrong_enum_sent_to_subgraph() {
    const SDL: &str = r###"
        enum join__Graph {
          GA
            @join__graph(
              name: "b"
              url: "https://b/graphql"
            )
          GB
            @join__graph(
              name: "a"
              url: "https://a/graphql"
            )
        }

        type Query {
          order: Order @join__field(graph: GA)
          doStuff(input: SomeInput!): String! @join__field(graph: GB)
        }

        enum Order {
          ASC
          DESC
        }

        enum Dummy {
          DESCOPE
        }

        input SomeInput {
          dummy: Dummy!
          token: String!
        }
        "###;

    runtime().block_on(async move {
        let fetcher = MockFetch::default().with_responses("a", vec![json!({"data": {"doStuff": "Hi!"}})]);
        let engine = Engine::builder()
            .with_federated_sdl(SDL)
            .with_mock_fetcher(fetcher.clone())
            .build()
            .await;

        let response = engine
            .post(
                r#"
                query RequestUserToken {
                    doStuff(
                        input: {
                            token: "<token>"
                            dummy: DESCOPE
                        }
                    )
                }
                "#,
            )
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "doStuff": "Hi!"
          }
        }
        "###);

        let requests = fetcher.drain_received_requests().collect::<Vec<_>>();
        insta::with_settings!({ sort_maps => true}, {
            insta::assert_json_snapshot!(requests, @r#"
            [
              [
                "a",
                {
                  "body": {
                    "query": "query($var0: SomeInput!) {\n  doStuff(input: $var0)\n}\n",
                    "operationName": null,
                    "variables": {
                      "var0": {
                        "dummy": "DESCOPE",
                        "token": "<token>"
                      }
                    },
                    "extensions": {}
                  },
                  "headers": [
                    [
                      "accept",
                      "application/graphql-response+json; charset=utf-8, application/json; charset=utf-8"
                    ],
                    [
                      "content-length",
                      "127"
                    ],
                    [
                      "content-type",
                      "application/json"
                    ]
                  ]
                }
              ]
            ]
            "#)
        });
    });
}

#[test]
fn gb7323_join_field_may_not_be_present() {
    const SDL: &str = r###"
    schema
      @link(url: "https://specs.apollo.dev/link/v1.0")
      @link(url: "https://specs.apollo.dev/join/v0.3", for: EXECUTION)
    {
      query: Query
    }

    directive @join__enumValue(graph: join__Graph!) repeatable on ENUM_VALUE

    directive @join__field(graph: join__Graph, requires: join__FieldSet, provides: join__FieldSet, type: String, external: Boolean, override: String, usedOverridden: Boolean) repeatable on FIELD_DEFINITION | INPUT_FIELD_DEFINITION

    directive @join__graph(name: String!, url: String!) on ENUM_VALUE

    directive @join__implements(graph: join__Graph!, interface: String!) repeatable on OBJECT | INTERFACE

    directive @join__type(graph: join__Graph!, key: join__FieldSet, extension: Boolean! = false, resolvable: Boolean! = true, isInterfaceObject: Boolean! = false) repeatable on OBJECT | INTERFACE | UNION | ENUM | INPUT_OBJECT | SCALAR

    directive @join__unionMember(graph: join__Graph!, member: String!) repeatable on UNION

    directive @link(url: String, as: String, for: link__Purpose, import: [link__Import]) repeatable on SCHEMA

    scalar join__FieldSet

    enum join__Graph {
      NAME @join__graph(name: "name", url: "http://localhost:4200/name")
    }

    scalar link__Import

    enum link__Purpose {
      SECURITY
      EXECUTION
    }
    type Product
      @join__type(graph: NAME)
    {
      id: ID!
      name: String
    }

    type Query
      @join__type(graph: NAME)
    {
      product: Product
      products: [Product]
    }
    "###;

    runtime().block_on(async move {
        let fetcher = MockFetch::default().with_responses("localhost", vec![json!({"data": {"product": {"id": "1"}}})]);
        let engine = Engine::builder()
            .with_federated_sdl(SDL)
            .with_mock_fetcher(fetcher.clone())
            .build()
            .await;

        let response = engine
            .post(
                r#"
                query { product { id } }
                "#,
            )
            .await;

        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "product": {
              "id": "1"
            }
          }
        }
        "###);
    });
}
