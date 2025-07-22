use graphql_mocks::FakeGithubSchema;
use integration_tests::{gateway::Gateway, runtime};

#[test]
fn multiple_operations_without_providing_operation_name_in_request() {
    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(FakeGithubSchema::default())
            .build()
            .await;

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

    insta::assert_json_snapshot!(response, @r#"
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
    "#);
}

#[test]
fn multiple_operations() {
    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(FakeGithubSchema::default())
            .build()
            .await;

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
            .operation_name("First")
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
        let engine = Gateway::builder()
            .with_subgraph(FakeGithubSchema::default())
            .build()
            .await;

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
        let engine = Gateway::builder()
            .with_subgraph(FakeGithubSchema::default())
            .build()
            .await;

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
            .operation_name("Third")
            .await
    });

    insta::assert_json_snapshot!(response, @r#"
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
    "#);
}
