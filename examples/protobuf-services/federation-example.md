# GraphQL Federation Example

This document demonstrates how the protobuf services work together in a federated GraphQL schema using the `@lookup` and `@key` directives.

## Federation Architecture

The three services form a federated graph where entities can be resolved across service boundaries:

```
┌─────────────────┐    @lookup     ┌─────────────────┐    @lookup     ┌─────────────────┐
│  Products       │──────────────▶│  Parts          │──────────────▶│  Locations      │
│  Service        │                │  Service        │                │  Service        │
│                 │◀──────────────│                 │◀──────────────│                 │
│ @key(id)        │    @lookup     │ @key(id)        │    @lookup     │ @key(id)        │
│ @lookup batch   │                │ @lookup batch   │                │ @lookup batch   │
└─────────────────┘                └─────────────────┘                └─────────────────┘
```

### Entity Keys and Lookups

### Products Service
- **Entity**: `Product @key(fields: "id")`
- **Batch Lookup**: `BatchGetProducts @lookup`
- **Relationship Lookups**: `GetProductParts @lookup`, `GetProductsForPart @lookup`
- **Entity Derivation**: `warehouse_id @derive` creates automatic Location entity reference

### Parts Service  
- **Entity**: `Part @key(fields: "id")`
- **Batch Lookup**: `BatchGetParts @lookup`
- **Entity Derivation**: `warehouse_id @derive` creates automatic Location entity reference

### Locations Service
- **Entity**: `Location @key(fields: "id")`
- **Batch Lookup**: `BatchGetLocations @lookup`

## Cross-Service Relationships

The services are connected through foreign key relationships with automatic entity derivation:

1. **Product → Location**: `Product.warehouse_id @derive` automatically creates Location entity reference
2. **Part → Location**: `Part.warehouse_id @derive` automatically creates Location entity reference  
3. **ProductPart → Product**: `ProductPart.product_id` references `Product.id`
4. **ProductPart → Part**: `ProductPart.part_id` references `Part.id`

## Example Federated Queries

### 1. Product with Warehouse Information

```graphql
query ProductWithWarehouse {
  # Start with products from Products service
  products {
    id
    name
    sku
    price
    warehouse_id
    
    # Federation automatically resolves warehouse via @lookup
    # Using BatchGetLocations with warehouse_id
    warehouse {
      id
      name
      address
      city
      manager_name
      is_active
    }
  }
}
```

**Federation Execution Plan:**
1. Query `products` from Products service
2. The `warehouse_id @derive` directive automatically creates Location entity references
3. Use `BatchGetLocations @lookup` to resolve warehouse entities
4. Merge location data into product results

### 2. Product Bill of Materials

```graphql
query ProductBillOfMaterials($productId: ID!) {
  # Get product parts relationship
  productParts(productId: $productId) {
    product_id
    part_id
    quantity_required
    
    # Resolve actual part details via @lookup
    part {
      id
      part_number
      name
      cost
      supplier
      quantity_available
      
      # Resolve part's warehouse via @lookup
      warehouse {
        id
        name
        address
        capacity
      }
    }
    
    # Resolve product details via @lookup
    product {
      id
      name
      sku
      price
    }
  }
}
```

**Federation Execution Plan:**
1. Query `GetProductParts` from Products service
2. Collect `part_id` values from product parts
3. Use `BatchGetParts @lookup` to resolve part details
4. Collect `warehouse_id` values from parts
5. Use `BatchGetLocations @lookup` to resolve warehouse details
6. If needed, use `BatchGetProducts @lookup` for product details
7. Merge all data into final response

### 3. Warehouse Inventory Overview

```graphql
query WarehouseInventory($warehouseId: ID!) {
  # Get warehouse details
  warehouse: location(id: $warehouseId) {
    id
    name
    address
    capacity
    is_active
    
    # Find all products stored at this warehouse
    products(warehouseId: $warehouseId) {
      id
      name
      sku
      quantity_in_stock
      
      # Get parts for each product
      parts {
        part_id
        quantity_required
        
        part {
          id
          part_number
          name
          quantity_available
          is_critical
        }
      }
    }
    
    # Find all parts stored at this warehouse
    parts(warehouseId: $warehouseId) {
      id
      part_number
      name
      quantity_available
      is_critical
      
      # Find which products use this part
      usedInProducts {
        product_id
        quantity_required
        
        product {
          id
          name
          sku
        }
      }
    }
  }
}
```

### 4. Complete Product Catalog with Dependencies

```graphql
query ProductCatalog {
  products {
    id
    name
    sku
    description
    price
    quantity_in_stock
    category
    
    # Product's warehouse
    warehouse {
      id
      name
      address
      city
      state
      manager_name
      contact_phone
    }
    
    # Required parts with their details
    parts {
      quantity_required
      
      part {
        id
        part_number
        name
        description
        cost
        supplier
        quantity_available
        category
        is_critical
        
        # Part's warehouse (may be different from product's)
        warehouse {
          id
          name
          address
          city
          state
        }
      }
    }
  }
}
```

## Federation Benefits

### 1. Efficient Data Loading
- **Batch Resolution**: `@lookup` directives on methods enable automatic batching
- **Entity Derivation**: `@derive` directives on fields automatically create entity references
- **N+1 Prevention**: Multiple entity lookups are batched into single calls
- **Optimized Queries**: Only requested fields are resolved

### 2. Service Autonomy
- **Independent Deployment**: Each service can be deployed separately
- **Schema Evolution**: Services can evolve their schemas independently
- **Technology Diversity**: Different services can use different technologies

### 3. Data Consistency
- **Single Source of Truth**: Each entity is owned by one service
- **Referential Integrity**: Foreign keys maintain relationships
- **Eventual Consistency**: Services can sync data as needed

## Query Optimization Examples

### Before Federation (Multiple Round Trips)
```javascript
// Client makes multiple requests
const products = await client.query('{ products { id warehouse_id } }');
const warehouseIds = products.map(p => p.warehouse_id);
const warehouses = await client.query(`{ locations(ids: [${warehouseIds}]) { id name } }`);

// Client manually joins data
const result = products.map(product => ({
  ...product,
  warehouse: warehouses.find(w => w.id === product.warehouse_id)
}));
```

### With Federation (Single Request)
```graphql
query {
  products {
    id
    warehouse_id
    warehouse {  # Automatically resolved via @lookup
      id
      name
    }
  }
}
```

## Implementation Notes

### Service Configuration
Each service needs to be configured with:
1. **Generated GraphQL Schema**: From protoc-gen-grafbase-subgraph
2. **gRPC Client Configuration**: To call other services for @lookup
3. **Federation Metadata**: Entity keys and lookup capabilities

### Gateway Configuration
The federation gateway needs:
1. **Schema Composition**: Combine all service schemas
2. **Query Planning**: Route queries to appropriate services
3. **Entity Resolution**: Handle @lookup directive execution

### Error Handling
- **Partial Failures**: Handle cases where some entities can't be resolved
- **Timeout Management**: Set appropriate timeouts for cross-service calls
- **Graceful Degradation**: Return partial results when possible

## Performance Considerations

### Caching Strategy
- **Entity Caching**: Cache frequently accessed entities
- **Batch Size Limits**: Prevent overly large batch requests
- **Cache Invalidation**: Coordinate cache updates across services

### Monitoring
- **Query Complexity**: Monitor and limit complex federated queries
- **Service Health**: Track health of all federation participants
- **Performance Metrics**: Measure query resolution times

This federation setup enables powerful cross-service queries while maintaining service boundaries and optimizing for performance.