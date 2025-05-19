use integration_tests::{
    gateway::{DockerSubgraph, Gateway},
    runtime,
};

#[test]
fn with_valid_certificate_and_identity() {
    let config = indoc::indoc! {r#"
        [subgraphs.mtls-test-subgraph.mtls]
        root.certificate = "data/mtls-subgraph/certs/ca-cert.pem"
        identity = "data/mtls-subgraph/certs/client-identity.pem"
    "#};

    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_docker_subgraph(DockerSubgraph::Mtls)
            .with_toml_config(config)
            .build()
            .await;

        engine.post("query { hello }").await
    });

    insta::assert_json_snapshot!(response.body, @r#"
    {
      "data": {
        "hello": "Hello, world"
      }
    }
    "#);
}

#[test]
fn with_valid_certificate_and_invalid_identity() {
    let config = indoc::indoc! {r#"
        [subgraphs.mtls-test-subgraph.mtls]
        root.certificate = "data/mtls-subgraph/certs/ca-cert.pem"
    "#};

    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_docker_subgraph(DockerSubgraph::Mtls)
            .with_toml_config(config)
            .build()
            .await;

        engine.post("query { hello }").await
    });

    insta::assert_json_snapshot!(response.body, @r#"
    {
      "data": null,
      "errors": [
        {
          "message": "Request to subgraph 'mtls-test-subgraph' failed.",
          "locations": [
            {
              "line": 1,
              "column": 9
            }
          ],
          "path": [
            "hello"
          ],
          "extensions": {
            "code": "SUBGRAPH_REQUEST_ERROR"
          }
        }
      ]
    }
    "#);
}

#[test]
fn with_invalid_certificate_and_valid_identity() {
    let config = indoc::indoc! {r#"
        [subgraphs.mtls-test-subgraph.mtls]
        identity = "data/mtls-subgraph/certs/client-identity.pem"
    "#};

    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_docker_subgraph(DockerSubgraph::Mtls)
            .with_toml_config(config)
            .build()
            .await;

        engine.post("query { hello }").await
    });

    insta::assert_json_snapshot!(response.body, @r#"
    {
      "data": null,
      "errors": [
        {
          "message": "Request to subgraph 'mtls-test-subgraph' failed.",
          "locations": [
            {
              "line": 1,
              "column": 9
            }
          ],
          "path": [
            "hello"
          ],
          "extensions": {
            "code": "SUBGRAPH_REQUEST_ERROR"
          }
        }
      ]
    }
    "#);
}

#[test]
fn with_accept_invalid_certs() {
    let config = indoc::indoc! {r#"
        [subgraphs.mtls-test-subgraph.mtls]
        identity = "data/mtls-subgraph/certs/client-identity.pem"
        accept_invalid_certs = true
    "#};

    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_docker_subgraph(DockerSubgraph::Mtls)
            .with_toml_config(config)
            .build()
            .await;

        engine.post("query { hello }").await
    });

    insta::assert_json_snapshot!(response.body, @r#"
    {
      "data": {
        "hello": "Hello, world"
      }
    }
    "#);
}
