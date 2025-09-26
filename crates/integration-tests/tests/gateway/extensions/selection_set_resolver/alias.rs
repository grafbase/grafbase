use integration_tests::{gateway::Gateway, runtime};

#[test]
fn root_field_aliases() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "echo-selection-set",
                r#"
                extend schema
                    @link(url: "selection-set-resolver-015", import: ["@init"])
                    @init

                type Book {
                    id: ID!
                    title: String!
                    author: Author!
                    pages: Int
                    published: Boolean
                    genres: [String!]
                }

                type Author {
                    id: ID!
                    name: String!
                    bio: String
                    books: [Book!]
                }

                type Query {
                    book(id: ID!): Book
                    books: [Book!]
                    author(id: ID!): Author
                    authors: [Author!]
                }
                "#,
            )
            .with_extension("selection-set-resolver-015")
            .build()
            .await;

        // Simple query with nested objects
        let response = engine
            .post(
                r#"
                query {
                    a: book(id: "a") {
                        id
                        pages
                    }
                    b: book(id: "b") {
                        id
                        title
                    }
                }
                "#,
            )
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "a": null,
            "b": null
          },
          "errors": [
            {
              "message": "MyError",
              "locations": [
                {
                  "line": 3,
                  "column": 21
                }
              ],
              "path": [
                "a"
              ],
              "extensions": {
                "selectionSet": {
                  "alias": "a",
                  "arguments": {
                    "id": "a"
                  },
                  "id": "Query.book",
                  "selectionSet": {
                    "fields": [
                      {
                        "arguments": {},
                        "id": "Book.id"
                      },
                      {
                        "arguments": {},
                        "id": "Book.pages"
                      }
                    ],
                    "requiresTypename": false
                  }
                },
                "code": "EXTENSION_ERROR"
              }
            },
            {
              "message": "MyError",
              "locations": [
                {
                  "line": 7,
                  "column": 21
                }
              ],
              "path": [
                "b"
              ],
              "extensions": {
                "selectionSet": {
                  "alias": "b",
                  "arguments": {
                    "id": "b"
                  },
                  "id": "Query.book",
                  "selectionSet": {
                    "fields": [
                      {
                        "arguments": {},
                        "id": "Book.id"
                      },
                      {
                        "arguments": {},
                        "id": "Book.title"
                      }
                    ],
                    "requiresTypename": false
                  }
                },
                "code": "EXTENSION_ERROR"
              }
            }
          ]
        }
        "#);
    })
}
