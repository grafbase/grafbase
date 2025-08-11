# Protobuf Services Implementation

This directory contains three microservices implemented in different languages that demonstrate cross-service entity references:

## Services

### 1. Products Service (TypeScript) - Port 50051
- Manages products and their relationships to parts
- References locations via `warehouse_id`
- Tracks product-part relationships with quantities

### 2. Parts Service (Go) - Port 50052
- Manages component parts inventory
- References locations via `warehouse_id`
- Tracks supplier information and criticality

### 3. Locations Service (Rust) - Port 50053
- Manages warehouse locations
- Referenced by both products and parts

## Cross-Service References

The services contain hardcoded data with the following entity relationships:

### Locations (warehouses):
- `loc-001`: Seattle Distribution Center
- `loc-002`: Portland Warehouse
- `loc-003`: San Francisco Hub

### Products:
- `prod-001`: Mountain Bike Pro (stored at `loc-001`)
- `prod-002`: Electric City Cruiser (stored at `loc-002`)
- `prod-003`: Speed Demon Road Bike (stored at `loc-001`)

### Parts Distribution:
- Mountain bike parts primarily in Seattle (`loc-001`)
- E-bike components in Portland (`loc-002`)
- Specialized parts in San Francisco (`loc-003`)

### Product-Part Relationships:
- Mountain Bike requires: frame, 2 wheels, derailleur, handlebars, 2 brake sets
- E-bike requires: motor, battery, 2 wheels, handlebars, display
- Road bike requires: lightweight frame, 2 racing wheels, derailleur, drop bars, 2 brake sets

## Running the Services

### TypeScript (Products Service):
```bash
cd services/products-ts
npm install
npm start
```

### Go (Parts Service):
```bash
cd services/parts-go
go mod download
go run .
```

### Rust (Locations Service):
```bash
cd services/locations-rust
cargo build
cargo run
```
