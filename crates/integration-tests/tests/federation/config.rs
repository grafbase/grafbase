use engine_v2::Engine;
use graphql_mocks::FederatedAccountsSchema;
use integration_tests::{federation::EngineV2Ext, runtime};

#[test]
fn subgraph_url_override() {
    runtime().block_on(async {
        let subgraph_server = graphql_mocks::MockGraphQlServer::new(FederatedAccountsSchema).await;

        let engine = Engine::builder()
            .with_federated_sdl(
                r###"
            enum join__Graph {
                ACCOUNTS @join__graph(name: "accounts", url: "http://0.0.0.0:0/")
            }

            type User
                @join__type(graph: ACCOUNTS, key: "id")
            {
                id: ID!
                username: String! @join__field(graph: ACCOUNTS)
            }

            type Query {
                me: User! @join__field(graph: ACCOUNTS)
            }
            "###,
            )
            .with_toml_config(format!(
                r###"
                [subgraphs.accounts]
                url = "{}"
                "###,
                subgraph_server.url()
            ))
            .build()
            .await;

        let response = engine
            .post(
                r"
                query ExampleQuery {
                    me {
                        username
                    }
                }
                ",
            )
            .await;

        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "me": {
              "username": "Me"
            }
          }
        }"###);
    });
}
