use grafbase_sdk::test::TestGateway;

#[tokio::test]
async fn test_example() {
    let gateway = TestGateway::builder()
        .subgraph(
            r#"
            extend schema @link(url: "<self>", import: ["@geo"])

            type Query {
                commune(code: String): Commune @geo
            }

            type Commune {
                code: String
                codeRegion: String
                codeDepartement: String
                nom: String
                population: Int
            }
            "#,
        )
        .build()
        .await
        .unwrap();

    let response = gateway
        .query(r#"query { commune(code: "75101") { nom codeRegion codeDepartement } }"#)
        .send()
        .await;

    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "commune": {
          "nom": "Paris 1er Arrondissement",
          "codeRegion": "11",
          "codeDepartement": "75"
        }
      }
    }
    "#);
}
