use graphql_mocks::dynamic::DynamicSchema;
use integration_tests::{gateway::Gateway, runtime};
use serde_json::json;

use crate::gateway::extensions::selection_set_resolver::StaticSelectionSetResolverExt;

#[test]
fn generate_the_correct_lookup_field_shape() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                    type Query {
                        users: [User!]!
                    }

                    type User @key(fields: "id") {
                        id: Int!
                        age: Int
                    }
                    "#,
                )
                .with_resolver("Query", "users", json!([{"id":3,"age":13},{"id":1,"age":12}]))
                .into_subgraph("gql"),
            )
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "static-1.0.0", import: ["@init"])
                    @link(url: "https://specs.grafbase.com/composite-schema/v1", import: ["@lookup", "@key"])
                    @init

                type Query {
                    userLookup(ids: [Int!]!): [User!]! @lookup
                }

                type User @key(fields: "id") {
                    id: Int!
                    name: String!
                    blogs: BlogConnection
                }

                type BlogConnection
                {
                    edges: [BlogEdge!]!
                }

                type BlogEdge
                {
                    node: Blog!
                }

                type Blog
                {
                    id: Int!
                }
                "#,
            )
            .with_extension(StaticSelectionSetResolverExt::json(json!([
               {
                  "id":3,
                  "name":"Pentti",
                  "blogs":{
                     "edges":[],
                     "pageInfo":{
                        "endCursor":"todo",
                        "hasNextPage":false,
                        "startCursor":"todo",
                        "hasPreviousPage":false
                     }
                  }
               },
               {
                  "id":1,
                  "name":"Musti",
                  "blogs":{
                     "edges":[
                        {
                           "node":{
                              "id":1
                           },
                           "cursor":"todo"
                        },
                        {
                           "node":{
                              "id":2
                           },
                           "cursor":"todo"
                        }
                     ],
                     "pageInfo":{
                        "endCursor":"todo",
                        "hasNextPage":false,
                        "startCursor":"todo",
                        "hasPreviousPage":false
                     }
                  }
               }
            ])))
            .build()
            .await;

        let response = engine
            .post(
                r#"
                query {
                  users {
                    id
                    name
                    age
                    blogs { edges { node { id } } }
                  }
                }
                "#,
            )
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "users": [
              {
                "id": 3,
                "name": "Pentti",
                "age": 13,
                "blogs": {
                  "edges": []
                }
              },
              {
                "id": 1,
                "name": "Musti",
                "age": 12,
                "blogs": {
                  "edges": [
                    {
                      "node": {
                        "id": 1
                      }
                    },
                    {
                      "node": {
                        "id": 2
                      }
                    }
                  ]
                }
              }
            ]
          }
        }
        "#);
    })
}
