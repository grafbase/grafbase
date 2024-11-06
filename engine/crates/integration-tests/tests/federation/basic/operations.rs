use engine_v2::Engine;
use graphql_mocks::FakeGithubSchema;
use integration_tests::{
    federation::{EngineV2Ext, GraphQlRequest},
    runtime,
};

#[test]
fn multiple_operations_without_providing_operation_name_in_request() {
    let response = runtime().block_on(async move {
        let engine = Engine::builder().with_subgraph(FakeGithubSchema).build().await;

        engine
            .post(
                r"
                    query First {
                        allBotPullRequests {
                            title
                        }
                    }

                    query Second {
                        allBotPullRequests {
                            title
                            checks
                        }
                    }
                    ",
            )
            .await
    });

    insta::assert_json_snapshot!(response, @r###"
    {
      "errors": [
        {
          "message": "Missing operation name.",
          "extensions": {
            "code": "OPERATION_PARSING_ERROR"
          }
        }
      ]
    }
    "###);
}

#[test]
fn multiple_operations() {
    let response = runtime().block_on(async move {
        let engine = Engine::builder().with_subgraph(FakeGithubSchema).build().await;

        engine
            .post(GraphQlRequest {
                query: r"
                    query First {
                        allBotPullRequests {
                            title
                        }
                    }

                    query Second {
                        allBotPullRequests {
                            title
                            checks
                        }
                    }
                    "
                .to_string(),
                operation_name: Some("First".to_string()),
                variables: None,
                extensions: None,
                doc_id: None,
            })
            .await
    });

    insta::assert_json_snapshot!(response, @r###"
    {
      "data": {
        "allBotPullRequests": [
          {
            "title": "Creating the thing"
          },
          {
            "title": "Some bot PR"
          }
        ]
      }
    }
    "###);
}

#[test]
fn only_one_named_operation() {
    let response = runtime().block_on(async move {
        let engine = Engine::builder().with_subgraph(FakeGithubSchema).build().await;

        engine
            .post(
                r"
                    query First {
                        allBotPullRequests {
                            title
                        }
                    }
                    ",
            )
            .await
    });

    insta::assert_json_snapshot!(response, @r###"
    {
      "data": {
        "allBotPullRequests": [
          {
            "title": "Creating the thing"
          },
          {
            "title": "Some bot PR"
          }
        ]
      }
    }
    "###);
}

#[test]
fn unknown_operation_name() {
    let response = runtime().block_on(async move {
        let engine = Engine::builder().with_subgraph(FakeGithubSchema).build().await;

        engine
            .post(GraphQlRequest {
                query: r"
                    query First {
                        allBotPullRequests {
                            title
                        }
                    }

                    query Second {
                        allBotPullRequests {
                            title
                            checks
                        }
                    }
                    "
                .to_string(),
                operation_name: Some("Third".to_string()),
                variables: None,
                extensions: None,
                doc_id: None,
            })
            .await
    });

    insta::assert_json_snapshot!(response, @r###"
    {
      "errors": [
        {
          "message": "Unknown operation named 'Third'.",
          "extensions": {
            "code": "OPERATION_PARSING_ERROR"
          }
        }
      ]
    }
    "###);
}
