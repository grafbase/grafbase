# Protocol Buffer Services Example

This directory contains three interconnected Protocol Buffer service definitions that demonstrate how to design microservices with cross-service relationships and intermediate relationship messages. The services include GraphQL directives for entity resolution and federation capabilities using the `protoc-gen-grafbase-subgraph` plugin.

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

### 2. Parts Service (`parts.proto`)
Manages component parts inventory.

**Key Fields:**
- `id`: Unique part identifier
- `part_number`: External part number for reference
- `warehouse_id`: References the location where the part is stored
- `quantity_available`: Current inventory level

**Key Methods:**
- `GetPart` / `BatchGetParts`: Retrieve part information

### 3. Locations Service (`locations.proto`)
Manages warehouse and storage locations.

**Key Fields:**
- `id`: Unique location identifier (referenced as `warehouse_id`)
- Address and contact information
- `capacity` and `is_active` status

**Key Methods:**
- `GetLocation` / `BatchGetLocations`: Retrieve location information

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

## Usage Examples

### Get Bill of Materials for a Product
1. Call `ProductService.GetProduct(id)` to get product details
2. Call `ProductService.GetProductParts(product_id)` to get parts with quantities
3. Call `PartService.BatchGetParts(part_ids)` to get detailed part information

### Find Products Using a Specific Part
1. Call `ProductService.GetProductsForPart(part_id)` to get product relationships
2. Call `ProductService.BatchGetProducts(product_ids)` to get product details

### Get Complete Product Information
1. Get product: `ProductService.GetProduct(product_id)`
2. Get warehouse: `LocationService.GetLocation(warehouse_id)`
3. Get parts list: `ProductService.GetProductParts(product_id)`
4. Get part details: `PartService.BatchGetParts(part_ids)`

## Key Features

- **Quantity Management**: `ProductPart.quantity_required` specifies exactly how many units of each part are needed
- **Batch Operations**: All services support efficient batch retrieval by IDs with `@lookup` directives for entity resolution
- **Clean Separation**: Each service manages its primary domain with minimal cross-references
- **Intermediate Relationships**: Product-part relationships handled via dedicated message type
- **GraphQL Federation**: Entities marked with `@key` directives enable cross-service entity resolution
- **Entity Resolution**: `@lookup` directives on batch methods enable efficient federated queries

## GraphQL Directives

The protobuf definitions include GraphQL directives for federation and entity resolution:

### Entity Keys
- `Product`, `Part`, and `Location` entities are marked with `@key(fields: "id")`
- These enable cross-service entity resolution in federated schemas

### Lookup Operations  
- `BatchGetProducts`, `BatchGetParts`, and `BatchGetLocations` methods have `@lookup` directives
- `GetProductParts` and `GetProductsForPart` methods also have `@lookup` directives
- These enable efficient entity resolution in federated GraphQL schemas
- All services default to Query fields using `default_to_query_fields = true`

### Entity Derivation
- `warehouse_id` fields in `Product` and `Part` have `@derive` directives
- This automatically creates entity references to the `Location` service
- `ProductPart` uses composite key `@key(fields: "product_id part_id")` for relationship management

## Compilation

```bash
# For Go
protoc --go_out=. --go-grpc_out=. *.proto

# For Python
protoc --python_out=. --grpc_python_out=. *.proto

# For GraphQL subgraph generation
protoc --grafbase-subgraph_out=. *.proto -I .
```

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
    # This will use @lookup to resolve parts via BatchGetParts
    parts {
      id
      name
      # This will use @lookup to resolve location via BatchGetLocations  
      warehouse {
        id
        name
        address
      }
    }
    # This will use @lookup to resolve location via BatchGetLocations
    warehouse {
      id
      name
    }
  }
}
```

The `@lookup` directives enable the GraphQL gateway to automatically batch and resolve cross-service entity references.

## Running the Services

This example includes three microservices implemented in different languages. You can run them individually or using Docker Compose.

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

### Running with Docker Compose

For easier deployment, use Docker Compose to run all services:

```bash
cd services
docker-compose up --build
```

This will start all three services with the following port mappings:
- Products Service: `localhost:50051`
- Parts Service: `localhost:50052`
- Locations Service: `localhost:50053`

### Testing the Services with gRPC Calls

Use `grpcurl` or any gRPC client to test the services. Here are example calls for each service:

#### Products Service Examples
```bash
# Get a single product
grpcurl -plaintext -d '{"id": "prod-001"}' localhost:50051 products.ProductService/GetProduct

# Get multiple products
grpcurl -plaintext -d '{"ids": ["prod-001", "prod-002"]}' localhost:50051 products.ProductService/BatchGetProducts

# Get parts for a product
grpcurl -plaintext -d '{"product_id": "prod-001"}' localhost:50051 products.ProductService/GetProductParts

# Get products using a specific part
grpcurl -plaintext -d '{"part_id": "part-001"}' localhost:50051 products.ProductService/GetProductsForPart
```

#### Parts Service Examples
```bash
# Get a single part
grpcurl -plaintext -d '{"id": "part-001"}' localhost:50052 parts.PartService/GetPart

# Get multiple parts
grpcurl -plaintext -d '{"ids": ["part-001", "part-002", "part-003"]}' localhost:50052 parts.PartService/BatchGetParts
```

#### Locations Service Examples
```bash
# Get a single location
grpcurl -plaintext -d '{"id": "loc-001"}' localhost:50053 locations.LocationService/GetLocation

# Get multiple locations
grpcurl -plaintext -d '{"ids": ["loc-001", "loc-002"]}' localhost:50053 locations.LocationService/BatchGetLocations
```

### Verifying Cross-Service References

To verify that cross-service references are working correctly, follow these test scenarios:

#### 1. Complete Product Information Chain
```bash
# Step 1: Get a product
grpcurl -plaintext -d '{"id": "prod-001"}' localhost:50051 products.ProductService/GetProduct

# Step 2: Use the warehouse_id from the response to get location details
grpcurl -plaintext -d '{"id": "loc-001"}' localhost:50053 locations.LocationService/GetLocation

# Step 3: Get parts for the product
grpcurl -plaintext -d '{"product_id": "prod-001"}' localhost:50051 products.ProductService/GetProductParts

# Step 4: Use part IDs from the response to get part details
grpcurl -plaintext -d '{"ids": ["part-001", "part-002"]}' localhost:50052 parts.PartService/BatchGetParts
```

#### 2. Verify Part-to-Warehouse References
```bash
# Get part details (includes warehouse_id)
grpcurl -plaintext -d '{"id": "part-001"}' localhost:50052 parts.PartService/GetPart

# Use the warehouse_id to verify location exists
grpcurl -plaintext -d '{"id": "loc-001"}' localhost:50053 locations.LocationService/GetLocation
```

#### 3. Test Entity Relationships
The services contain pre-populated data with these relationships:

**Locations:**
- `loc-001`: Seattle Distribution Center
- `loc-002`: Portland Warehouse
- `loc-003`: San Francisco Hub

**Products:**
- `prod-001`: Mountain Bike Pro (warehouse: `loc-001`)
- `prod-002`: Electric City Cruiser (warehouse: `loc-002`)
- `prod-003`: Speed Demon Road Bike (warehouse: `loc-001`)

**Expected Cross-References:**
- Products and Parts reference Locations via `warehouse_id`
- ProductParts link Products to Parts with `quantity_required`
- All entity IDs should resolve across services

If all services respond correctly and the referenced IDs exist across services, the cross-service relationships are working properly.