# =============================
# ===== Sanity check ==========
# =============================
POST http://localhost:5000/graphql
Content-Type: application/json
```graphql
query {
  __typename
}
```

HTTP 200
[Asserts]
jsonpath "$.data.__typename" == "Query"

# =============================
# ===== Get single product ====
# =============================
POST http://localhost:5000/graphql
Content-Type: application/json
```graphql
query {
  products_ProductService_GetProduct(input: { id: "prod-001" }) {
    product {
      id
      name
      price
      quantity_in_stock
    }
  }
}
```

HTTP 200
[Asserts]
jsonpath "$.data.products_ProductService_GetProduct.product.id" == "prod-001"
jsonpath "$.data.products_ProductService_GetProduct.product.name" exists

# =============================
# ===== Get single location ===
# =============================
POST http://localhost:5000/graphql
Content-Type: application/json
```graphql
query {
  locations_LocationService_GetLocation(input: { id: "loc-001" }) {
    location {
      id
      name
      city
      country
    }
  }
}
```

HTTP 200
[Asserts]
jsonpath "$.data.locations_LocationService_GetLocation.location.id" == "loc-001"
jsonpath "$.data.locations_LocationService_GetLocation.location.name" exists

# =============================
# ===== Get single part =======
# =============================
POST http://localhost:5000/graphql
Content-Type: application/json
```graphql
query {
  parts_PartService_GetPart(input: { id: "part-001" }) {
    part {
      id
      name
      cost
      supplier
    }
  }
}
```

HTTP 200
[Asserts]
jsonpath "$.data.parts_PartService_GetPart.part.id" == "part-001"
jsonpath "$.data.parts_PartService_GetPart.part.name" exists

# =============================
# ===== Complex query =========
# =============================
POST http://localhost:5000/graphql
Content-Type: application/json
```graphql
query {
  products_ProductService_SearchProducts(input: {}) {
    products {
      name
      id
      warehouse_id
      warehouse {
        name
        address
        city
      }
      parts {
        product_parts {
          part {
            id
            category
            name
          }
        }
      }
    }
  }
}
```

HTTP 200
[Asserts]
jsonpath "$.data.products_ProductService_SearchProducts.products" isCollection

# =============================
# ===== Batch operations ======
# =============================
POST http://localhost:5000/graphql
Content-Type: application/json
```graphql
query {
  products_ProductService_BatchGetProducts(input: { ids: ["prod-001", "PROD002"] }) {
    products {
      id
      name
      sku
    }
  }
}
```

HTTP 200
[Asserts]
jsonpath "$.data.products_ProductService_BatchGetProducts.products" isCollection

# =============================
# ===== Search with filters ===
# =============================
POST http://localhost:5000/graphql
Content-Type: application/json
```graphql
query {
  parts_PartService_SearchParts(input: { category: "Electronics" }) {
    parts {
      id
      name
      category
      warehouse_id
      warehouse {
        name
        city
      }
    }
  }
}
```

HTTP 200
[Asserts]
jsonpath "$.data.parts_PartService_SearchParts.parts" isCollection

# =============================
# ===== Product parts query ===
# =============================
POST http://localhost:5000/graphql
Content-Type: application/json
```graphql
query {
  products_ProductService_GetProductParts(input: { product_id: "prod-001" }) {
    product_parts {
      part_id
      quantity_required
      part {
        name
        cost
        supplier
      }
    }
  }
}
```

HTTP 200
[Asserts]
jsonpath "$.data.products_ProductService_GetProductParts.product_parts" isCollection

# =============================
# ===== Nested resolution =====
# =============================
POST http://localhost:5000/graphql
Content-Type: application/json
```graphql
query {
  products_ProductService_SearchProducts(input: { category: "E-Bikes" }) {
    products {
      id
      name
      warehouse {
        id
        name
        manager_name
        capacity
      }
    }
  }
}
```

HTTP 200
[Asserts]
jsonpath "$.data.products_ProductService_SearchProducts.products" isCollection

# =============================
# ===== Multiple queries ======
# =============================
POST http://localhost:5000/graphql
Content-Type: application/json
```graphql
query {
  product1: products_ProductService_GetProduct(input: { id: "prod-001" }) {
    product {
      id
      name
    }
  }
  location1: locations_LocationService_GetLocation(input: { id: "loc-001" }) {
    location {
      id
      name
    }
  }
  part1: parts_PartService_GetPart(input: { id: "part-001" }) {
    part {
      id
      name
    }
  }
}
```

HTTP 200
[Asserts]
jsonpath "$.data.product1.product.id" == "prod-001"
jsonpath "$.data.location1.location.id" == "loc-001"
jsonpath "$.data.part1.part.id" == "part-001"
