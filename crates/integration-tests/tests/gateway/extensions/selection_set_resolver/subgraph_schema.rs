use integration_tests::{gateway::Gateway, runtime};

#[test]
fn receive_subgraph_schema() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "echo-schema",
                r#"
                extend schema
                    @link(url: "selection-set-resolver-014-1.0.0", import: ["@init", "@meta"]) @init @meta(data: "Schema")
                    @link(url: "https://specs.grafbase.com/composite-schema/v1", import: ["@key"])

                # Scalar types
                scalar JSON
                scalar DateTime @specifiedBy(url: "https://datatracker.ietf.org/doc/html/rfc3339")
                scalar URL @meta(data: "URL")

                # Object types
                type Query @meta(data: "Query") {
                    test: JSON @meta(data: "Query.test")
                    user(id: ID! @meta(data: "Query.user.id")): User
                    products(filter: ProductFilter): [Product!]
                    searchItems(term: String!): [SearchResult!]
                }

                type User implements Node @key(fields: "id") {
                    id: ID!
                    name: String!
                    email: String!
                    orders: [Order!]
                    createdAt: DateTime
                }

                type Product implements Node @key(fields: "id") {
                    id: ID!
                    name: String!
                    description: String
                    price: Float!
                    category: Category!
                    tags: [String!]
                    inStock: Boolean!
                    attributes: ProductAttributes
                }

                type ProductAttributes {
                    color: String
                    weight: Float
                }

                type Order {
                    id: ID!
                    items: [OrderItem!]!
                    total: Float!
                    status: OrderStatus!
                    createdAt: DateTime!
                }

                type OrderItem {
                    product: Product!
                    quantity: Int!
                    price: Float!
                }

                # Interface types
                interface Node @meta(data: "Node") {
                    id: ID!
                }

                interface Timestamped {
                    createdAt: DateTime!
                    updatedAt: DateTime
                }

                # Union type
                union SearchResult @meta(data: "SearchResult") = User | Product

                # Enum types
                enum OrderStatus @meta(data: "OrderStatus") {
                    PENDING
                    PROCESSING
                    SHIPPED
                    DELIVERED
                    CANCELLED
                }

                enum Category {
                    ELECTRONICS
                    CLOTHING
                    BOOKS
                    HOME
                    BEAUTY
                    SPORTS
                    TOYS
                }

                # Input types
                input ProductFilter @meta(data: "ProductFilter") {
                    name: String @meta(data: "ProductFilter.name")
                    minPrice: Float
                    maxPrice: Float
                    categories: [Category!]
                    inStock: Boolean
                }

                input UserInput {
                    name: String!
                    email: String!
                    profileBio: String
                }
                "#,
            )
            .with_subgraph_sdl(
                "echo-config",
                r#"
                extend schema @link(url: "https://specs.apollo.dev/federation/v2.0", import: ["@key"])

                type Query {
                    config: Config!
                }

                type Config {
                    name: String!
                    version: String!
                    features: [Feature!]!
                }

                type Feature {
                    name: String!
                    enabled: Boolean!
                    description: String
                }

                extend type User @key(fields: "id") {
                    id: ID! @external
                    settings: UserSettings
                }

                type UserSettings {
                    theme: String!
                    notifications: Boolean!
                    language: String!
                }
                "#,
            )
            .with_subgraph_sdl(
                "other",
                r#"
                extend schema @link(url: "https://specs.apollo.dev/federation/v2.0", import: ["@key"])

                type Query {
                    metrics: Metrics!
                }

                type Metrics {
                    visitors: Int!
                    pageViews: Int!
                    conversionRate: Float!
                }

                extend type Product @key(fields: "id") {
                    id: ID! @external
                    reviews: [Review!]
                    rating: Float
                }

                type Review {
                    id: ID!
                    author: String!
                    text: String!
                    rating: Int!
                    createdAt: String!
                }
                "#,
            )
            .with_extension("selection-set-resolver-014")
            .build()
            .await;

        let response = engine.post(r#"query { test }"#).await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "test": [
              {
                "directives": [
                  {
                    "arguments": {},
                    "name": "init"
                  },
                  {
                    "arguments": {
                      "data": "Schema"
                    },
                    "name": "meta"
                  }
                ],
                "mutation": null,
                "name": "echo-schema",
                "query": "Query",
                "subscription": null,
                "typeDefinitions": [
                  {
                    "Boolean": {
                      "directives": [],
                      "kind": "SCALAR",
                      "specifiedByUrl": null
                    }
                  },
                  {
                    "Category": {
                      "directives": [],
                      "kind": "ENUM",
                      "name": "Category",
                      "values": [
                        {
                          "directives": [],
                          "name": "ELECTRONICS"
                        },
                        {
                          "directives": [],
                          "name": "CLOTHING"
                        },
                        {
                          "directives": [],
                          "name": "BOOKS"
                        },
                        {
                          "directives": [],
                          "name": "HOME"
                        },
                        {
                          "directives": [],
                          "name": "BEAUTY"
                        },
                        {
                          "directives": [],
                          "name": "SPORTS"
                        },
                        {
                          "directives": [],
                          "name": "TOYS"
                        }
                      ]
                    }
                  },
                  {
                    "DateTime": {
                      "directives": [],
                      "kind": "SCALAR",
                      "specifiedByUrl": null
                    }
                  },
                  {
                    "Float": {
                      "directives": [],
                      "kind": "SCALAR",
                      "specifiedByUrl": null
                    }
                  },
                  {
                    "ID": {
                      "directives": [],
                      "kind": "SCALAR",
                      "specifiedByUrl": null
                    }
                  },
                  {
                    "Int": {
                      "directives": [],
                      "kind": "SCALAR",
                      "specifiedByUrl": null
                    }
                  },
                  {
                    "JSON": {
                      "directives": [],
                      "kind": "SCALAR",
                      "specifiedByUrl": null
                    }
                  },
                  {
                    "Node": {
                      "directives": [
                        {
                          "arguments": {
                            "data": "Node"
                          },
                          "name": "meta"
                        }
                      ],
                      "fields": [
                        {
                          "arguments": [],
                          "directives": [],
                          "name": "id",
                          "type": {
                            "definition": "ID",
                            "wrapping": [
                              "NON_NULL"
                            ]
                          }
                        }
                      ],
                      "interfaces": [],
                      "kind": "INTERFACE"
                    }
                  },
                  {
                    "Order": {
                      "directives": [],
                      "fields": [
                        {
                          "arguments": [],
                          "directives": [],
                          "name": "createdAt",
                          "type": {
                            "definition": "DateTime",
                            "wrapping": [
                              "NON_NULL"
                            ]
                          }
                        },
                        {
                          "arguments": [],
                          "directives": [],
                          "name": "id",
                          "type": {
                            "definition": "ID",
                            "wrapping": [
                              "NON_NULL"
                            ]
                          }
                        },
                        {
                          "arguments": [],
                          "directives": [],
                          "name": "items",
                          "type": {
                            "definition": "OrderItem",
                            "wrapping": [
                              "NON_NULL",
                              "LIST",
                              "NON_NULL"
                            ]
                          }
                        },
                        {
                          "arguments": [],
                          "directives": [],
                          "name": "status",
                          "type": {
                            "definition": "OrderStatus",
                            "wrapping": [
                              "NON_NULL"
                            ]
                          }
                        },
                        {
                          "arguments": [],
                          "directives": [],
                          "name": "total",
                          "type": {
                            "definition": "Float",
                            "wrapping": [
                              "NON_NULL"
                            ]
                          }
                        }
                      ],
                      "interfaces": [],
                      "kind": "OBJECT"
                    }
                  },
                  {
                    "OrderItem": {
                      "directives": [],
                      "fields": [
                        {
                          "arguments": [],
                          "directives": [],
                          "name": "price",
                          "type": {
                            "definition": "Float",
                            "wrapping": [
                              "NON_NULL"
                            ]
                          }
                        },
                        {
                          "arguments": [],
                          "directives": [],
                          "name": "product",
                          "type": {
                            "definition": "Product",
                            "wrapping": [
                              "NON_NULL"
                            ]
                          }
                        },
                        {
                          "arguments": [],
                          "directives": [],
                          "name": "quantity",
                          "type": {
                            "definition": "Int",
                            "wrapping": [
                              "NON_NULL"
                            ]
                          }
                        }
                      ],
                      "interfaces": [],
                      "kind": "OBJECT"
                    }
                  },
                  {
                    "OrderStatus": {
                      "directives": [
                        {
                          "arguments": {
                            "data": "OrderStatus"
                          },
                          "name": "meta"
                        }
                      ],
                      "kind": "ENUM",
                      "name": "OrderStatus",
                      "values": [
                        {
                          "directives": [],
                          "name": "PENDING"
                        },
                        {
                          "directives": [],
                          "name": "PROCESSING"
                        },
                        {
                          "directives": [],
                          "name": "SHIPPED"
                        },
                        {
                          "directives": [],
                          "name": "DELIVERED"
                        },
                        {
                          "directives": [],
                          "name": "CANCELLED"
                        }
                      ]
                    }
                  },
                  {
                    "Product": {
                      "directives": [],
                      "fields": [
                        {
                          "arguments": [],
                          "directives": [],
                          "name": "attributes",
                          "type": {
                            "definition": "ProductAttributes",
                            "wrapping": []
                          }
                        },
                        {
                          "arguments": [],
                          "directives": [],
                          "name": "category",
                          "type": {
                            "definition": "Category",
                            "wrapping": [
                              "NON_NULL"
                            ]
                          }
                        },
                        {
                          "arguments": [],
                          "directives": [],
                          "name": "description",
                          "type": {
                            "definition": "String",
                            "wrapping": []
                          }
                        },
                        {
                          "arguments": [],
                          "directives": [],
                          "name": "id",
                          "type": {
                            "definition": "ID",
                            "wrapping": [
                              "NON_NULL"
                            ]
                          }
                        },
                        {
                          "arguments": [],
                          "directives": [],
                          "name": "inStock",
                          "type": {
                            "definition": "Boolean",
                            "wrapping": [
                              "NON_NULL"
                            ]
                          }
                        },
                        {
                          "arguments": [],
                          "directives": [],
                          "name": "name",
                          "type": {
                            "definition": "String",
                            "wrapping": [
                              "NON_NULL"
                            ]
                          }
                        },
                        {
                          "arguments": [],
                          "directives": [],
                          "name": "price",
                          "type": {
                            "definition": "Float",
                            "wrapping": [
                              "NON_NULL"
                            ]
                          }
                        },
                        {
                          "arguments": [],
                          "directives": [],
                          "name": "tags",
                          "type": {
                            "definition": "String",
                            "wrapping": [
                              "NON_NULL",
                              "LIST"
                            ]
                          }
                        }
                      ],
                      "interfaces": [
                        "Node"
                      ],
                      "kind": "OBJECT"
                    }
                  },
                  {
                    "ProductAttributes": {
                      "directives": [],
                      "fields": [
                        {
                          "arguments": [],
                          "directives": [],
                          "name": "color",
                          "type": {
                            "definition": "String",
                            "wrapping": []
                          }
                        },
                        {
                          "arguments": [],
                          "directives": [],
                          "name": "weight",
                          "type": {
                            "definition": "Float",
                            "wrapping": []
                          }
                        }
                      ],
                      "interfaces": [],
                      "kind": "OBJECT"
                    }
                  },
                  {
                    "ProductFilter": {
                      "directives": [
                        {
                          "arguments": {
                            "data": "ProductFilter"
                          },
                          "name": "meta"
                        }
                      ],
                      "inputFields": [
                        {
                          "directives": [
                            {
                              "arguments": {
                                "data": "ProductFilter.name"
                              },
                              "name": "meta"
                            }
                          ],
                          "name": "name",
                          "type": {
                            "definition": "String",
                            "wrapping": []
                          }
                        },
                        {
                          "directives": [],
                          "name": "inStock",
                          "type": {
                            "definition": "Boolean",
                            "wrapping": []
                          }
                        },
                        {
                          "directives": [],
                          "name": "minPrice",
                          "type": {
                            "definition": "Float",
                            "wrapping": []
                          }
                        },
                        {
                          "directives": [],
                          "name": "maxPrice",
                          "type": {
                            "definition": "Float",
                            "wrapping": []
                          }
                        },
                        {
                          "directives": [],
                          "name": "categories",
                          "type": {
                            "definition": "Category",
                            "wrapping": [
                              "NON_NULL",
                              "LIST"
                            ]
                          }
                        }
                      ],
                      "kind": "INPUT_OBJECT"
                    }
                  },
                  {
                    "Query": {
                      "directives": [
                        {
                          "arguments": {
                            "data": "Query"
                          },
                          "name": "meta"
                        }
                      ],
                      "fields": [
                        {
                          "arguments": [
                            {
                              "directives": [],
                              "name": "filter",
                              "type": {
                                "definition": "ProductFilter",
                                "wrapping": []
                              }
                            }
                          ],
                          "directives": [],
                          "name": "products",
                          "type": {
                            "definition": "Product",
                            "wrapping": [
                              "NON_NULL",
                              "LIST"
                            ]
                          }
                        },
                        {
                          "arguments": [
                            {
                              "directives": [],
                              "name": "term",
                              "type": {
                                "definition": "String",
                                "wrapping": [
                                  "NON_NULL"
                                ]
                              }
                            }
                          ],
                          "directives": [],
                          "name": "searchItems",
                          "type": {
                            "definition": "SearchResult",
                            "wrapping": [
                              "NON_NULL",
                              "LIST"
                            ]
                          }
                        },
                        {
                          "arguments": [],
                          "directives": [
                            {
                              "arguments": {
                                "data": "Query.test"
                              },
                              "name": "meta"
                            }
                          ],
                          "name": "test",
                          "type": {
                            "definition": "JSON",
                            "wrapping": []
                          }
                        },
                        {
                          "arguments": [
                            {
                              "directives": [
                                {
                                  "arguments": {
                                    "data": "Query.user.id"
                                  },
                                  "name": "meta"
                                }
                              ],
                              "name": "id",
                              "type": {
                                "definition": "ID",
                                "wrapping": [
                                  "NON_NULL"
                                ]
                              }
                            }
                          ],
                          "directives": [],
                          "name": "user",
                          "type": {
                            "definition": "User",
                            "wrapping": []
                          }
                        }
                      ],
                      "interfaces": [],
                      "kind": "OBJECT"
                    }
                  },
                  {
                    "SearchResult": {
                      "directives": [
                        {
                          "arguments": {
                            "data": "SearchResult"
                          },
                          "name": "meta"
                        }
                      ],
                      "kind": "UNION",
                      "memberTypes": [
                        "User",
                        "Product"
                      ]
                    }
                  },
                  {
                    "String": {
                      "directives": [],
                      "kind": "SCALAR",
                      "specifiedByUrl": null
                    }
                  },
                  {
                    "Timestamped": {
                      "directives": [],
                      "fields": [
                        {
                          "arguments": [],
                          "directives": [],
                          "name": "createdAt",
                          "type": {
                            "definition": "DateTime",
                            "wrapping": [
                              "NON_NULL"
                            ]
                          }
                        },
                        {
                          "arguments": [],
                          "directives": [],
                          "name": "updatedAt",
                          "type": {
                            "definition": "DateTime",
                            "wrapping": []
                          }
                        }
                      ],
                      "interfaces": [],
                      "kind": "INTERFACE"
                    }
                  },
                  {
                    "URL": {
                      "directives": [
                        {
                          "arguments": {
                            "data": "URL"
                          },
                          "name": "meta"
                        }
                      ],
                      "kind": "SCALAR",
                      "specifiedByUrl": null
                    }
                  },
                  {
                    "User": {
                      "directives": [],
                      "fields": [
                        {
                          "arguments": [],
                          "directives": [],
                          "name": "createdAt",
                          "type": {
                            "definition": "DateTime",
                            "wrapping": []
                          }
                        },
                        {
                          "arguments": [],
                          "directives": [],
                          "name": "email",
                          "type": {
                            "definition": "String",
                            "wrapping": [
                              "NON_NULL"
                            ]
                          }
                        },
                        {
                          "arguments": [],
                          "directives": [],
                          "name": "id",
                          "type": {
                            "definition": "ID",
                            "wrapping": [
                              "NON_NULL"
                            ]
                          }
                        },
                        {
                          "arguments": [],
                          "directives": [],
                          "name": "name",
                          "type": {
                            "definition": "String",
                            "wrapping": [
                              "NON_NULL"
                            ]
                          }
                        },
                        {
                          "arguments": [],
                          "directives": [],
                          "name": "orders",
                          "type": {
                            "definition": "Order",
                            "wrapping": [
                              "NON_NULL",
                              "LIST"
                            ]
                          }
                        }
                      ],
                      "interfaces": [
                        "Node"
                      ],
                      "kind": "OBJECT"
                    }
                  },
                  {
                    "UserInput": {
                      "directives": [],
                      "inputFields": [
                        {
                          "directives": [],
                          "name": "name",
                          "type": {
                            "definition": "String",
                            "wrapping": [
                              "NON_NULL"
                            ]
                          }
                        },
                        {
                          "directives": [],
                          "name": "email",
                          "type": {
                            "definition": "String",
                            "wrapping": [
                              "NON_NULL"
                            ]
                          }
                        },
                        {
                          "directives": [],
                          "name": "profileBio",
                          "type": {
                            "definition": "String",
                            "wrapping": []
                          }
                        }
                      ],
                      "kind": "INPUT_OBJECT"
                    }
                  }
                ]
              }
            ]
          }
        }
        "#);
    });
}
