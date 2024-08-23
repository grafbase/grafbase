use engine_v2::Engine;
use integration_tests::{federation::EngineV2Ext as _, runtime};

const SDL: &str = r#"
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
    FST @join__graph(name: "fst", url: "http://does.not.exist")
}

type User
    @join__type(graph: FST, key: "id")
{
    id: ID!
    name: String @join__field(graph: FST) @deprecated(reason: "we have no name")
}

type Query {
    user: User @join__field(graph: FST)
}
"#;

#[test]
fn subgraph_not_reachable_does_not_leak_subgraph_url() {
    runtime().block_on(async move {
        let engine = Engine::builder().with_federated_sdl(SDL).build().await;

        let response = engine.post(r#"query { user { name } }"#).await;

        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "user": null
          },
          "errors": [
            {
              "message": "Request to subgraph 'fst' failed with: error sending request",
              "path": [
                "user"
              ],
              "extensions": {
                "code": "SUBGRAPH_REQUEST_ERROR"
              }
            }
          ]
        }
        "###);
    })
}
