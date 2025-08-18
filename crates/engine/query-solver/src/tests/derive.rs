use crate::assert_solving_snapshots;

const SCHEMA: &str = r#"
    type Product
      @join__type(graph: EXT)
    {
      authorId: ID!
      commentIds: [ID!]!
      inventoryCountryId: ID!
      inventoryWarehouseId: ID!
      inventoriesKeys: [InventoryKeys!]!
      code: String!
      id: ID!
      author: User! @composite__derive(graph: EXT)
      comments: [Comment!]! @composite__derive(graph: EXT)
      inventory: Inventory! @composite__derive(graph: EXT)
      inventories: [Inventory!]! @composite__derive(graph: EXT)
    }

    type User
        @join__type(graph: EXT, key: "id", resolvable: false)
    {
      id: ID!
    }

    type Comment
        @join__type(graph: EXT, key: "id", resolvable: false)
    {
      id: ID!
    }

    type InventoryKeys @join__type(graph: EXT) {
        countryId: ID!
        warehouseId: ID!
    }

    type Inventory
        @join__type(graph: EXT, key: "countryId warehouseId", resolvable: false)
    {
        countryId: ID!
        warehouseId: ID!
    }

    type Query
    {
      products: [Product!]! @join__field(graph: EXT)
    }

    enum join__Graph
    {
    EXT @join__graph(name: "ext", url: "http://localhost:8080")
    }
"#;

#[test]
fn single_id() {
    assert_solving_snapshots!(
        "single_id",
        SCHEMA,
        r#"
        query {
            products {
                author {
                    id
                }
            }
        }
        "#
    );
}

#[test]
fn composite_keys() {
    assert_solving_snapshots!(
        "composite_keys",
        SCHEMA,
        r#"
        query {
            products {
                 inventory {
                    countryId
                    warehouseId
                }
            }
        }
        "#
    );
}

#[test]
fn batch_single_id() {
    assert_solving_snapshots!(
        "batch_single_id",
        SCHEMA,
        r#"
        query {
            products {
                comments {
                    id
                }
            }
        }
        "#
    );
}

#[test]
fn batch_composite_keys() {
    assert_solving_snapshots!(
        "batch_composite_keys",
        SCHEMA,
        r#"
        query {
            products {
                inventories {
                    countryId
                    warehouseId
                }
            }
        }
        "#
    );
}
