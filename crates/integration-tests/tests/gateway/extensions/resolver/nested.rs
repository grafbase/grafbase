use integration_tests::{gateway::Gateway, runtime};

#[test]
fn nested_static_resolver() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "resolver", import: ["@resolve"])

                scalar JSON

                type Query {
                    me: User @resolve(data: { id: "1", name: "Josh" })
                }

                type User {
                    id: ID!
                    name: String!
                    friends: [User!]! @resolve(data: [{ id: "2", name: "Alice"}])
                }
                "#,
            )
            .with_extension(super::ResolverExt::echo_data())
            .build()
            .await;

        let response = engine.post("query { me { id name friends { id name } } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "me": {
              "id": "1",
              "name": "Josh",
              "friends": [
                {
                  "id": "2",
                  "name": "Alice"
                }
              ]
            }
          }
        }
        "#);
    })
}

#[test]
fn lookup_shouldnt_count_as_nested_resolver() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "resolver", import: ["@resolve"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@lookup", "@key"])

                scalar JSON

                type Query {
                    user(id: ID!): User @resolve(data: {}) @lookup
                    me: User @resolve(data: { id: "1", name: "Josh" })
                }

                type User @key(fields: "id") {
                    id: ID!
                    name: String!
                    friends: [User!]! @resolve(data: [{ id: "2", name: "Alice"}])
                }
                "#,
            )
            .with_extension(super::ResolverExt::echo_data())
            .build()
            .await;

        let response = engine.post("query { me { id name friends { id name } } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "me": {
              "id": "1",
              "name": "Josh",
              "friends": [
                {
                  "id": "2",
                  "name": "Alice"
                }
              ]
            }
          }
        }
        "#);
    })
}
