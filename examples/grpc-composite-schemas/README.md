# Protocol Buffer Services Example

This directory contains three interconnected Protocol Buffer service definitions that demonstrate how to design microservices with cross-service relationships and intermediate relationship messages. The services include GraphQL directives for entity resolution and federation capabilities using the `protoc-gen-grafbase-subgraph` plugin.

## Exposing the GraphQL federated graph composed from the gRPC services

You'll need the following binaries:

- `buf`
- `grafbase`
- `grafbase-gateway` (optional)

Generate subgraph schemas from protobuf definitions:

```bash
buf generate
```

That generates three subgraphs in `gen/graphql`.

You can now either follow the instructions below or directly start the development server with:

```bash
grafbase dev
```

If you want to use the gateway directly instead of the development server, compose the federated graph:

```bash
grafbase compose > federated.graphql
```

This works because `grafbase.toml` defines the subgraphs.

Alternatively, you can compose with the schema registry with calls to `grafbase publish`.

Now start the gateway directly:

```bash
grafbase-gateway --schema federated.graphql
```

## Services Overview

### 1. Products Service (`products.proto`)

Manages products and their relationships to parts with quantities.

**Key Messages:**

- `Product`: Core product information with `warehouse_id` reference
- `ProductPart`: Intermediate message linking products to parts with `quantity_required`

**Key Methods:**

- `GetProduct` / `BatchGetProducts`: Retrieve product information
- `GetProductParts`: Get all parts needed for a product with quantities
- `GetProductsForPart`: Find all products that use a specific part
- `SearchProducts`: Search products with optional filters (name, category, price range, warehouse, stock, SKU)

### 2. Parts Service (`parts.proto`)

Manages component parts inventory.

**Key Fields:**

- `id`: Unique part identifier
- `part_number`: External part number for reference
- `warehouse_id`: References the location where the part is stored
- `quantity_available`: Current inventory level

**Key Methods:**

- `GetPart` / `BatchGetParts`: Retrieve part information
- `SearchParts`: Search parts with optional filters (name, category, supplier, warehouse, quantity, cost range, critical status, part number)

### 3. Locations Service (`locations.proto`)

Manages warehouse and storage locations.

**Key Fields:**

- `id`: Unique location identifier (referenced as `warehouse_id`)
- Address and contact information
- `capacity` and `is_active` status

**Key Methods:**

- `GetLocation` / `BatchGetLocations`: Retrieve location information
- `SearchLocations`: Search locations with optional filters (name, city, state, country, capacity, active status, postal code, manager)

## Service Relationships

```
┌─────────────┐    warehouse_id     ┌─────────────┐
│   Product   │────────────────────▶│  Location   │
│             │                     │             │
│             │                     └─────────────┘
│             │                            ▲
│      ┌──────▼──────┐                     │
│      │ ProductPart │                     │ warehouse_id
│      │             │                     │
│      │quantity_    │              ┌─────────────┐
│      │ required    │              │    Part     │
│      │             │              │             │
│      └──────┬──────┘              │             │
│             │                     │             │
│             └────────────────────▶│             │
│                  part_id          └─────────────┘
└─────────────┘
```

## GraphQL Directives

The protobuf definitions include GraphQL directives for federation and entity resolution. They are based on the upcoming [composite schemas standard](https://github.com/graphql/composite-schemas-spec/) developed by the most important actors in the GraphQL federation ecosystem as a working group inside the GraphQL Foundation.

To get an overview of the available protobuf options, see the [protoc-gen-grafbase-subgraph README](https://github.com/grafbase/extensions/tree/main/cli/protoc-gen-grafbase-subgraph).

Here are the most options and directives that enable joins between services:

- The `@lookup` and `@key` directives define which fields the gateway can use to resolve an entity in a service based on a key (unique identifier). The example services include both single entity and batch lookups.
- The `@derive` directive lets you define virtual fields on a type, so you can build a type not defined in your service from one of its keys. For example, if your Product type has a warehouse_id field referencing a location, you can declaratively define a Location type to enable joins between services:

  ```graphql
  type Product {
      sku: String!
      warehouse_id: ID!
      warehouse: Location! @derive
  }

  type Location @key(fields: "id") {
      id: ID!
  }
  ```

  The `Location` type here will be merged with the `Location` types in other subgraphs (services), and you can now send queries like `{ products { warehouse { address } } }`, assuming one of the other services defines the `address` field on `Location`.

- The `grafbase.graphql.join_field` allows joins based on gRPC methods of arbitrary shapes, not only lookups (lookups take ids and return the entities directly).

  ```proto
  option (grafbase.graphql.join_field) = {
      name: "parts",
      service: "products.ProductService",
      method: "GetProductParts",
      require: "{ product_id: id }"
  };
  ```

  You can see examples of the `@grpcMethod` directive in action in the code generated by protoc-gen-grafbase-subgraph.

## Example Data Flow

A bicycle product might have:
- `ProductPart{product_id: "bike123", part_id: "wheel456", quantity_required: 2}`
- `ProductPart{product_id: "bike123", part_id: "frame789", quantity_required: 1}`

This clearly shows that building one bicycle requires 2 wheels and 1 frame.

## GraphQL Federation Example

With the `@lookup` directives, a federated GraphQL query can efficiently resolve entities across services:

```graphql
query {
  products {
    id
    name
    parts {
      id
      name
      warehouse {
        id
        name
        address
      }
    }
    warehouse {
      id
      name
    }
  }
}

{
  products_ProductService_SearchProducts(input: {}) {
    products {
      name
      id
      warehouse_id
      # This will use @lookup to resolve locations via BatchGetLocations
      warehouse {
        name
        address
        city
      }
      # This field calls out to `GetProductParts` explicitly, not through an @lookup directive (see products.proto).
      parts {
        product_parts {
          # Batch lookup to the parts service
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

## Running the Services

This example includes three microservices implemented in different languages. You can run them individually or using Docker Compose.

### Running with Docker Compose

For easier deployment, use Docker Compose to run all services:

```bash
docker-compose up
```

This will start all three services with the following port mappings:
- Products Service: `localhost:50051`
- Parts Service: `localhost:50052`
- Locations Service: `localhost:50053`
- Grafbase Gateway: `localhost:5000`

### Running Services Individually (Native)

#### Prerequisites
- Node.js 18+ (for Products service)
- Go 1.19+ (for Parts service)
- Rust 1.70+ (for Locations service)

#### 1. Products Service (TypeScript) - Port 50051
```bash
cd services/products-ts
npm install
npm start
```

#### 2. Parts Service (Go) - Port 50052
```bash
cd services/parts-go
go mod download
go run .
```

#### 3. Locations Service (Rust) - Port 50053
```bash
cd services/locations-rust
cargo build
cargo run
```
