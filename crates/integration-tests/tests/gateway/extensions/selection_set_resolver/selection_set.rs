use integration_tests::{gateway::Gateway, runtime};

#[test]
fn type_conditions() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "echo-selection-set",
                r#"
                extend schema
                    @link(url: "selection-set-resolver-015-1.0.0", import: ["@init"])
                    @init

                interface Character {
                    id: ID!
                    name: String!
                }

                type Human implements Character {
                    id: ID!
                    name: String!
                    homePlanet: String
                }

                type Droid implements Character {
                    id: ID!
                    name: String!
                    primaryFunction: String
                }

                union SearchResult = Human | Droid

                type Query {
                    character(id: ID!): Character
                    search(text: String): [SearchResult!]
                    humans: [Human!]
                    droids: [Droid!]
                }
                "#,
            )
            .with_extension("selection-set-resolver-015")
            .build()
            .await;

        // Query with interface type
        let response = engine
            .post(
                r#"
                query {
                    character(id: "1000") {
                        name
                        ... on Human {
                            homePlanet
                        }
                        ... on Droid {
                            primaryFunction
                        }
                    }
                }
                "#,
            )
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "character": null
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
                "character"
              ],
              "extensions": {
                "selectionSet": {
                  "arguments": {
                    "id": "1000"
                  },
                  "id": "Query.character",
                  "selectionSet": {
                    "fields": [
                      {
                        "arguments": {},
                        "id": "Character.name"
                      },
                      {
                        "arguments": {},
                        "id": "Human.homePlanet"
                      },
                      {
                        "arguments": {},
                        "id": "Droid.primaryFunction"
                      }
                    ],
                    "requiresTypename": true
                  }
                },
                "code": "EXTENSION_ERROR"
              }
            }
          ]
        }
        "#);

        // Query with union type
        let response = engine
            .post(
                r#"
                query {
                    search(text: "droid") {
                        __typename
                        ... on Human {
                            name
                        }
                        ... on Droid {
                            name
                        }
                        ... on Human {
                            homePlanet
                        }
                        ... on Droid {
                            primaryFunction
                        }
                    }
                }
                "#,
            )
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "search": null
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
                "search"
              ],
              "extensions": {
                "selectionSet": {
                  "arguments": {
                    "text": "droid"
                  },
                  "id": "Query.search",
                  "selectionSet": {
                    "fields": [
                      {
                        "arguments": {},
                        "id": "Human.name"
                      },
                      {
                        "arguments": {},
                        "id": "Human.homePlanet"
                      },
                      {
                        "arguments": {},
                        "id": "Droid.name"
                      },
                      {
                        "arguments": {},
                        "id": "Droid.primaryFunction"
                      }
                    ],
                    "requiresTypename": true
                  }
                },
                "code": "EXTENSION_ERROR"
              }
            }
          ]
        }
        "#);

        // Complex query with multiple type conditions and nested fragments
        let response = engine
            .post(
                r#"
                query {
                    humans {
                        ...humanFields
                    }
                    droids {
                        ...droidFields
                    }
                    search(text: "all") {
                        ...searchFields
                    }
                }

                fragment humanFields on Human {
                    id
                    name
                    homePlanet
                }

                fragment droidFields on Droid {
                    id
                    name
                    primaryFunction
                }

                fragment searchFields on SearchResult {
                    __typename
                    ... on Character {
                        id
                    }
                    ... on Human {
                        homePlanet
                    }
                    ... on Droid {
                        primaryFunction
                    }
                    ... on Character {
                        name
                    }
                }
                "#,
            )
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "humans": null,
            "droids": null,
            "search": null
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
                "humans"
              ],
              "extensions": {
                "selectionSet": {
                  "arguments": {},
                  "id": "Query.humans",
                  "selectionSet": {
                    "fields": [
                      {
                        "arguments": {},
                        "id": "Human.id"
                      },
                      {
                        "arguments": {},
                        "id": "Human.name"
                      },
                      {
                        "arguments": {},
                        "id": "Human.homePlanet"
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
                  "line": 6,
                  "column": 21
                }
              ],
              "path": [
                "droids"
              ],
              "extensions": {
                "selectionSet": {
                  "arguments": {},
                  "id": "Query.droids",
                  "selectionSet": {
                    "fields": [
                      {
                        "arguments": {},
                        "id": "Droid.id"
                      },
                      {
                        "arguments": {},
                        "id": "Droid.name"
                      },
                      {
                        "arguments": {},
                        "id": "Droid.primaryFunction"
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
                  "line": 9,
                  "column": 21
                }
              ],
              "path": [
                "search"
              ],
              "extensions": {
                "selectionSet": {
                  "arguments": {
                    "text": "all"
                  },
                  "id": "Query.search",
                  "selectionSet": {
                    "fields": [
                      {
                        "arguments": {},
                        "id": "Character.id"
                      },
                      {
                        "arguments": {},
                        "id": "Character.name"
                      },
                      {
                        "arguments": {},
                        "id": "Human.homePlanet"
                      },
                      {
                        "arguments": {},
                        "id": "Droid.primaryFunction"
                      }
                    ],
                    "requiresTypename": true
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

#[test]
fn complex_arguments() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "echo-selection-set",
                r#"
                extend schema
                    @link(url: "selection-set-resolver-015-1.0.0", import: ["@init"])
                    @init

                input BookFilter {
                    title: String
                    minPages: Int
                    maxPages: Int
                    genres: [String!]
                    published: Boolean
                }

                input AuthorFilter {
                    name: String
                    country: String
                }

                input Pagination {
                    limit: Int!
                    offset: Int!
                }

                input SortInput {
                    field: String!
                    direction: String! # "ASC" or "DESC"
                }

                type Book {
                    id: ID!
                    title: String!
                    author: Author!
                    pages: Int
                    published: Boolean
                    genres: [String!]
                    price: Float
                }

                type Author {
                    id: ID!
                    name: String!
                    bio: String
                    country: String
                    books(filter: BookFilter, pagination: Pagination, sort: SortInput): [Book!]
                }

                type Query {
                    books(
                        filter: BookFilter, 
                        pagination: Pagination, 
                        sort: [SortInput!]
                    ): [Book!]
 
                    authors(
                        filter: AuthorFilter, 
                        pagination: Pagination, 
                        sort: SortInput
                    ): [Author!]

                    searchBooks(
                        query: String!, 
                        genres: [String!], 
                        priceRange: [Float!], 
                        pagination: Pagination
                    ): [Book!]

                    booksByAuthor(
                        authorId: ID!, 
                        includeUnpublished: Boolean = false, 
                        pagination: Pagination
                    ): [Book!]
                }
                "#,
            )
            .with_extension("selection-set-resolver-015")
            .build()
            .await;

        // Query with complex input objects as arguments
        let response = engine
            .post(
                r#"
                query {
                    books(
                        filter: {
                            title: "GraphQL in Action",
                            minPages: 200,
                            genres: ["Programming", "Computer Science"],
                            published: true
                        },
                        pagination: { limit: 10, offset: 0 },
                        sort: [
                            { field: "title", direction: "ASC" },
                            { field: "pages", direction: "DESC" }
                        ]
                    ) {
                        id
                        title
                        pages
                        author {
                            name
                            country
                        }
                    }
                }
                "#,
            )
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "books": null
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
                "books"
              ],
              "extensions": {
                "selectionSet": {
                  "arguments": {
                    "filter": {
                      "genres": [
                        "Programming",
                        "Computer Science"
                      ],
                      "minPages": 200,
                      "published": true,
                      "title": "GraphQL in Action"
                    },
                    "pagination": {
                      "limit": 10,
                      "offset": 0
                    },
                    "sort": [
                      {
                        "direction": "ASC",
                        "field": "title"
                      },
                      {
                        "direction": "DESC",
                        "field": "pages"
                      }
                    ]
                  },
                  "id": "Query.books",
                  "selectionSet": {
                    "fields": [
                      {
                        "arguments": {},
                        "id": "Book.id"
                      },
                      {
                        "arguments": {},
                        "id": "Book.title"
                      },
                      {
                        "arguments": {},
                        "id": "Book.pages"
                      },
                      {
                        "arguments": {},
                        "id": "Book.author",
                        "selectionSet": {
                          "fields": [
                            {
                              "arguments": {},
                              "id": "Author.name"
                            },
                            {
                              "arguments": {},
                              "id": "Author.country"
                            }
                          ],
                          "requiresTypename": false
                        }
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

        // Query with multiple operations using different argument types
        let response = engine
            .post(
                r#"
                query {
                    searchBooks(
                        query: "programming language",
                        genres: ["Programming", "Education"],
                        priceRange: [15.99, 49.99],
                        pagination: { limit: 5, offset: 0 }
                    ) {
                        id
                        title
                        price
                    }

                    booksByAuthor(
                        authorId: "auth123",
                        includeUnpublished: true,
                        pagination: { limit: 10, offset: 0 }
                    ) {
                        id
                        title
                        published
                    }
 
                    authors(
                        filter: { country: "USA" },
                        pagination: { limit: 20, offset: 0 },
                        sort: { field: "name", direction: "ASC" }
                    ) {
                        id
                        name
                        books(
                            filter: { minPages: 300 },
                            pagination: { limit: 5, offset: 0 },
                            sort: { field: "title", direction: "ASC" }
                        ) {
                            title
                            pages
                        }
                    }
                }
                "#,
            )
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "searchBooks": null,
            "booksByAuthor": null,
            "authors": null
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
                "searchBooks"
              ],
              "extensions": {
                "selectionSet": {
                  "arguments": {
                    "genres": [
                      "Programming",
                      "Education"
                    ],
                    "pagination": {
                      "limit": 5,
                      "offset": 0
                    },
                    "priceRange": [
                      15.99,
                      49.99
                    ],
                    "query": "programming language"
                  },
                  "id": "Query.searchBooks",
                  "selectionSet": {
                    "fields": [
                      {
                        "arguments": {},
                        "id": "Book.id"
                      },
                      {
                        "arguments": {},
                        "id": "Book.title"
                      },
                      {
                        "arguments": {},
                        "id": "Book.price"
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
                  "line": 14,
                  "column": 21
                }
              ],
              "path": [
                "booksByAuthor"
              ],
              "extensions": {
                "selectionSet": {
                  "arguments": {
                    "authorId": "auth123",
                    "includeUnpublished": true,
                    "pagination": {
                      "limit": 10,
                      "offset": 0
                    }
                  },
                  "id": "Query.booksByAuthor",
                  "selectionSet": {
                    "fields": [
                      {
                        "arguments": {},
                        "id": "Book.id"
                      },
                      {
                        "arguments": {},
                        "id": "Book.title"
                      },
                      {
                        "arguments": {},
                        "id": "Book.published"
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
                  "line": 24,
                  "column": 21
                }
              ],
              "path": [
                "authors"
              ],
              "extensions": {
                "selectionSet": {
                  "arguments": {
                    "filter": {
                      "country": "USA"
                    },
                    "pagination": {
                      "limit": 20,
                      "offset": 0
                    },
                    "sort": {
                      "direction": "ASC",
                      "field": "name"
                    }
                  },
                  "id": "Query.authors",
                  "selectionSet": {
                    "fields": [
                      {
                        "arguments": {},
                        "id": "Author.id"
                      },
                      {
                        "arguments": {},
                        "id": "Author.name"
                      },
                      {
                        "arguments": {
                          "filter": {
                            "minPages": 300
                          },
                          "pagination": {
                            "limit": 5,
                            "offset": 0
                          },
                          "sort": {
                            "direction": "ASC",
                            "field": "title"
                          }
                        },
                        "id": "Author.books",
                        "selectionSet": {
                          "fields": [
                            {
                              "arguments": {},
                              "id": "Book.title"
                            },
                            {
                              "arguments": {},
                              "id": "Book.pages"
                            }
                          ],
                          "requiresTypename": false
                        }
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

#[test]
fn variables() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "echo-selection-set",
                r#"
                extend schema
                    @link(url: "selection-set-resolver-015-1.0.0", import: ["@init"])
                    @init

                input Filters {
                    genres: [String!]
                    includeUnpublished: Boolean = false
                }

                type Book {
                    id: ID!
                }

                type Query {
                    search(text: String!, filters: Filters, limit: Int = 100): Int
                }
                "#,
            )
            .with_extension("selection-set-resolver-015")
            .build()
            .await;

        let response = engine
            .post(
                r#"
                query($text: String!) {
                    search(text: $text) 
                }
                "#,
            )
            .variables(serde_json::json!({"text": "GraphQL in Action"}))
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "search": null
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
                "search"
              ],
              "extensions": {
                "selectionSet": {
                  "arguments": {
                    "limit": 100,
                    "text": "GraphQL in Action"
                  },
                  "id": "Query.search"
                },
                "code": "EXTENSION_ERROR"
              }
            }
          ]
        }
        "#);

        let response = engine
            .post(
                r#"
                query($text: String!, $filters: Filters, $limit: Int) {
                    search(text: $text, filters: $filters, limit: $limit)
                }
                "#,
            )
            .variables(serde_json::json!({"text": "GraphQL in Action"}))
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "search": null
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
                "search"
              ],
              "extensions": {
                "selectionSet": {
                  "arguments": {
                    "limit": 100,
                    "text": "GraphQL in Action"
                  },
                  "id": "Query.search"
                },
                "code": "EXTENSION_ERROR"
              }
            }
          ]
        }
        "#);

        let response = engine
            .post(
                r#"
                query($text: String!, $filters: Filters, $limit: Int) {
                    search(text: $text, filters: $filters, limit: $limit)
                }
                "#,
            )
            .variables(serde_json::json!({"text": "GraphQL in Action", "filters": null, "limit": null}))
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "search": null
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
                "search"
              ],
              "extensions": {
                "selectionSet": {
                  "arguments": {
                    "filters": null,
                    "limit": null,
                    "text": "GraphQL in Action"
                  },
                  "id": "Query.search"
                },
                "code": "EXTENSION_ERROR"
              }
            }
          ]
        }
        "#);

        let response = engine
            .post(
                r#"
                query($text: String!, $filters: Filters, $limit: Int) {
                    search(text: $text, filters: $filters, limit: $limit)
                }
                "#,
            )
            .variables(
                serde_json::json!({"text": "GraphQL in Action", "filters": {"genres": ["Thriller"]}, "limit": 90}),
            )
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "search": null
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
                "search"
              ],
              "extensions": {
                "selectionSet": {
                  "arguments": {
                    "filters": {
                      "genres": [
                        "Thriller"
                      ],
                      "includeUnpublished": false
                    },
                    "limit": 90,
                    "text": "GraphQL in Action"
                  },
                  "id": "Query.search"
                },
                "code": "EXTENSION_ERROR"
              }
            }
          ]
        }
        "#);

        let response = engine
            .post(
                r#"
                query($text: String!, $filters: Filters, $limit: Int) {
                    search(text: $text, filters: $filters, limit: $limit)
                }
                "#,
            )
            .variables(
                serde_json::json!({"text": "GraphQL in Action", "filters": {"genres": ["Thriller"], "includeUnpublished": null}, "limit": 90}),
            )
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "search": null
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
                "search"
              ],
              "extensions": {
                "selectionSet": {
                  "arguments": {
                    "filters": {
                      "genres": [
                        "Thriller"
                      ],
                      "includeUnpublished": null
                    },
                    "limit": 90,
                    "text": "GraphQL in Action"
                  },
                  "id": "Query.search"
                },
                "code": "EXTENSION_ERROR"
              }
            }
          ]
        }
        "#);

        let response = engine
            .post(
                r#"
                query($text: String!, $filters: Filters, $limit: Int) {
                    search(text: $text, filters: $filters, limit: $limit)
                }
                "#,
            )
            .variables(
                serde_json::json!({"text": "GraphQL in Action", "filters": {"genres": ["Thriller"], "includeUnpublished": true}, "limit": 90}),
            )
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "search": null
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
                "search"
              ],
              "extensions": {
                "selectionSet": {
                  "arguments": {
                    "filters": {
                      "genres": [
                        "Thriller"
                      ],
                      "includeUnpublished": true
                    },
                    "limit": 90,
                    "text": "GraphQL in Action"
                  },
                  "id": "Query.search"
                },
                "code": "EXTENSION_ERROR"
              }
            }
          ]
        }
        "#);

        let response = engine
            .post(
                r#"
                query($genres: [String!], $includeUnpublished: Boolean) {
                    search(text: "GraphQL", filters: {genres: $genres, includeUnpublished: $includeUnpublished}, limit: 90)
                }
                "#,
            )
            .variables(
                serde_json::json!({}),
            )
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "search": null
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
                "search"
              ],
              "extensions": {
                "selectionSet": {
                  "arguments": {
                    "filters": {
                      "includeUnpublished": false
                    },
                    "limit": 90,
                    "text": "GraphQL"
                  },
                  "id": "Query.search"
                },
                "code": "EXTENSION_ERROR"
              }
            }
          ]
        }
        "#);

        let response = engine
            .post(
                r#"
                query($genres: [String!], $includeUnpublished: Boolean) {
                    search(text: "GraphQL", filters: {genres: $genres, includeUnpublished: $includeUnpublished}, limit: 90)
                }
                "#,
            )
            .variables(
                serde_json::json!({"genres": null, "includeUnpublished": null}),
            )
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "search": null
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
                "search"
              ],
              "extensions": {
                "selectionSet": {
                  "arguments": {
                    "filters": {
                      "genres": null,
                      "includeUnpublished": null
                    },
                    "limit": 90,
                    "text": "GraphQL"
                  },
                  "id": "Query.search"
                },
                "code": "EXTENSION_ERROR"
              }
            }
          ]
        }
        "#);

        let response = engine
            .post(
                r#"
                query($genres: [String!], $includeUnpublished: Boolean) {
                    search(text: "GraphQL", filters: {genres: $genres, includeUnpublished: $includeUnpublished}, limit: 90)
                }
                "#,
            )
            .variables(
                serde_json::json!({"genres": ["Thriller"], "includeUnpublished": true}),
            )
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "search": null
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
                "search"
              ],
              "extensions": {
                "selectionSet": {
                  "arguments": {
                    "filters": {
                      "genres": [
                        "Thriller"
                      ],
                      "includeUnpublished": true
                    },
                    "limit": 90,
                    "text": "GraphQL"
                  },
                  "id": "Query.search"
                },
                "code": "EXTENSION_ERROR"
              }
            }
          ]
        }
        "#);
    })
}

#[test]
fn simple_object_selection_set() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "echo-selection-set",
                r#"
                extend schema
                    @link(url: "selection-set-resolver-015-1.0.0", import: ["@init"])
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
                    book(id: "1") {
                        id
                        title
                        pages
                        published
                        author {
                            id
                            name
                            bio
                        }
                        genres
                    }
                }
                "#,
            )
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "book": null
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
                "book"
              ],
              "extensions": {
                "selectionSet": {
                  "arguments": {
                    "id": "1"
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
                      },
                      {
                        "arguments": {},
                        "id": "Book.pages"
                      },
                      {
                        "arguments": {},
                        "id": "Book.published"
                      },
                      {
                        "arguments": {},
                        "id": "Book.author",
                        "selectionSet": {
                          "fields": [
                            {
                              "arguments": {},
                              "id": "Author.id"
                            },
                            {
                              "arguments": {},
                              "id": "Author.name"
                            },
                            {
                              "arguments": {},
                              "id": "Author.bio"
                            }
                          ],
                          "requiresTypename": false
                        }
                      },
                      {
                        "arguments": {},
                        "id": "Book.genres"
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

        // Query with multiple root fields and nested objects
        let response = engine
            .post(
                r#"
                query {
                    books {
                        id
                        title
                    }
                    authors {
                        id
                        name
                        books {
                            title
                            pages
                        }
                    }
                }
                "#,
            )
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "books": null,
            "authors": null
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
                "books"
              ],
              "extensions": {
                "selectionSet": {
                  "arguments": {},
                  "id": "Query.books",
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
                "authors"
              ],
              "extensions": {
                "selectionSet": {
                  "arguments": {},
                  "id": "Query.authors",
                  "selectionSet": {
                    "fields": [
                      {
                        "arguments": {},
                        "id": "Author.id"
                      },
                      {
                        "arguments": {},
                        "id": "Author.name"
                      },
                      {
                        "arguments": {},
                        "id": "Author.books",
                        "selectionSet": {
                          "fields": [
                            {
                              "arguments": {},
                              "id": "Book.title"
                            },
                            {
                              "arguments": {},
                              "id": "Book.pages"
                            }
                          ],
                          "requiresTypename": false
                        }
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

        // Query with aliases and nested objects
        let response = engine
            .post(
                r#"
                query {
                    bookInfo: book(id: "1") {
                        bookId: id
                        bookTitle: title
                        bookAuthor: author {
                            authorName: name
                        }
                    }
                    authorInfo: author(id: "1") {
                        authorId: id
                        authorName: name
                        authorBooks: books {
                            bookTitle: title
                        }
                    }
                }
                "#,
            )
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "bookInfo": null,
            "authorInfo": null
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
                "bookInfo"
              ],
              "extensions": {
                "selectionSet": {
                  "alias": "bookInfo",
                  "arguments": {
                    "id": "1"
                  },
                  "id": "Query.book",
                  "selectionSet": {
                    "fields": [
                      {
                        "alias": "bookId",
                        "arguments": {},
                        "id": "Book.id"
                      },
                      {
                        "alias": "bookTitle",
                        "arguments": {},
                        "id": "Book.title"
                      },
                      {
                        "alias": "bookAuthor",
                        "arguments": {},
                        "id": "Book.author",
                        "selectionSet": {
                          "fields": [
                            {
                              "alias": "authorName",
                              "arguments": {},
                              "id": "Author.name"
                            }
                          ],
                          "requiresTypename": false
                        }
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
                  "line": 10,
                  "column": 21
                }
              ],
              "path": [
                "authorInfo"
              ],
              "extensions": {
                "selectionSet": {
                  "alias": "authorInfo",
                  "arguments": {
                    "id": "1"
                  },
                  "id": "Query.author",
                  "selectionSet": {
                    "fields": [
                      {
                        "alias": "authorId",
                        "arguments": {},
                        "id": "Author.id"
                      },
                      {
                        "alias": "authorName",
                        "arguments": {},
                        "id": "Author.name"
                      },
                      {
                        "alias": "authorBooks",
                        "arguments": {},
                        "id": "Author.books",
                        "selectionSet": {
                          "fields": [
                            {
                              "alias": "bookTitle",
                              "arguments": {},
                              "id": "Book.title"
                            }
                          ],
                          "requiresTypename": false
                        }
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

#[test]
fn nested_type_conditions() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "echo-selection-set",
                r#"
                extend schema
                    @link(url: "selection-set-resolver-015-1.0.0", import: ["@init"])
                    @init

                interface Node {
                    id: ID!
                }

                interface Character {
                    id: ID!
                    name: String!
                    friends: [Character!]
                }

                type Human implements Character & Node {
                    id: ID!
                    name: String!
                    homePlanet: String
                    friends: [Character!]
                    starships: [Starship!]
                }

                type Droid implements Character & Node {
                    id: ID!
                    name: String!
                    primaryFunction: String
                    friends: [Character!]
                }

                type Starship implements Node {
                    id: ID!
                    name: String!
                    length: Float
                }

                type Query {
                    node(id: ID!): Node
                    hero: Character
                }
                "#,
            )
            .with_extension("selection-set-resolver-015")
            .build()
            .await;

        // Query with deeply nested type conditions
        let response = engine
            .post(
                r#"
                query {
                    hero {
                        name
                        ... on Human {
                            homePlanet
                            friends {
                                ... on Droid {
                                    primaryFunction
                                }
                            }
                            starships {
                                length
                            }
                        }
                        ... on Droid {
                            primaryFunction
                            friends {
                                name
                                ... on Human {
                                    homePlanet
                                }
                            }
                        }
                        ... on Human {
                            homePlanet
                            friends {
                                name
                            }
                            starships {
                                name
                            }
                        }
                        ... on Droid {
                            friends {
                                ...X
                            }
                        }
                    }
                }

                fragment X on Human {
                    starships {
                        name
                    }
                }
                "#,
            )
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "hero": null
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
                "hero"
              ],
              "extensions": {
                "selectionSet": {
                  "arguments": {},
                  "id": "Query.hero",
                  "selectionSet": {
                    "fields": [
                      {
                        "arguments": {},
                        "id": "Character.name"
                      },
                      {
                        "arguments": {},
                        "id": "Human.homePlanet"
                      },
                      {
                        "arguments": {},
                        "id": "Human.friends",
                        "selectionSet": {
                          "fields": [
                            {
                              "arguments": {},
                              "id": "Character.name"
                            },
                            {
                              "arguments": {},
                              "id": "Droid.primaryFunction"
                            }
                          ],
                          "requiresTypename": true
                        }
                      },
                      {
                        "arguments": {},
                        "id": "Human.starships",
                        "selectionSet": {
                          "fields": [
                            {
                              "arguments": {},
                              "id": "Starship.length"
                            },
                            {
                              "arguments": {},
                              "id": "Starship.name"
                            }
                          ],
                          "requiresTypename": false
                        }
                      },
                      {
                        "arguments": {},
                        "id": "Droid.primaryFunction"
                      },
                      {
                        "arguments": {},
                        "id": "Droid.friends",
                        "selectionSet": {
                          "fields": [
                            {
                              "arguments": {},
                              "id": "Character.name"
                            },
                            {
                              "arguments": {},
                              "id": "Human.homePlanet"
                            },
                            {
                              "arguments": {},
                              "id": "Human.starships",
                              "selectionSet": {
                                "fields": [
                                  {
                                    "arguments": {},
                                    "id": "Starship.name"
                                  }
                                ],
                                "requiresTypename": false
                              }
                            }
                          ],
                          "requiresTypename": true
                        }
                      }
                    ],
                    "requiresTypename": true
                  }
                },
                "code": "EXTENSION_ERROR"
              }
            }
          ]
        }
        "#);

        // Query with interface and multiple implementations
        let response = engine
            .post(
                r#"
                query {
                    node(id: "1000") {
                        id
                        ... on Character {
                            name
                            friends {
                                id
                                name
                            }
                        }
                        ... on Human {
                            homePlanet
                            starships {
                                id
                                name
                            }
                        }
                        ... on Droid {
                            primaryFunction
                        }
                        ... on Starship {
                            length
                        }
                    }
                }
                "#,
            )
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "node": null
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
                "node"
              ],
              "extensions": {
                "selectionSet": {
                  "arguments": {
                    "id": "1000"
                  },
                  "id": "Query.node",
                  "selectionSet": {
                    "fields": [
                      {
                        "arguments": {},
                        "id": "Node.id"
                      },
                      {
                        "arguments": {},
                        "id": "Character.name"
                      },
                      {
                        "arguments": {},
                        "id": "Character.friends",
                        "selectionSet": {
                          "fields": [
                            {
                              "arguments": {},
                              "id": "Character.id"
                            },
                            {
                              "arguments": {},
                              "id": "Character.name"
                            }
                          ],
                          "requiresTypename": false
                        }
                      },
                      {
                        "arguments": {},
                        "id": "Human.homePlanet"
                      },
                      {
                        "arguments": {},
                        "id": "Human.starships",
                        "selectionSet": {
                          "fields": [
                            {
                              "arguments": {},
                              "id": "Starship.id"
                            },
                            {
                              "arguments": {},
                              "id": "Starship.name"
                            }
                          ],
                          "requiresTypename": false
                        }
                      },
                      {
                        "arguments": {},
                        "id": "Droid.primaryFunction"
                      },
                      {
                        "arguments": {},
                        "id": "Starship.length"
                      }
                    ],
                    "requiresTypename": true
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
