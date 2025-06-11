# Grafbase Postgres Extension - Relay Pagination Demo

This demo application showcases Relay-style cursor-based pagination using the Grafbase Postgres extension GraphQL API. It demonstrates a clean, modern UI for exploring products, variants, and inventory data with proper pagination controls.

## Features

- Relay-style cursor-based pagination
- Navigation between related entities (Products → Variants → Inventory)
- Clean, responsive UI
- Demonstrates GraphQL pagination with `hasNextPage`, `hasPreviousPage`, `before`, `after` and cursors

## Prerequisites

- Node.js (14.x or later)
- A running instance of the Grafbase Postgres extension example

## Setup

1. Install the Grafbase CLI:

```bash
curl -fsSL https://grafbase.com/downloads/cli | bash
```

2. Make sure you have the Grafbase Postgres extension example running:

```bash
docker compose up -d
cd grafbase
grafbase dev
```

3. Install dependencies for the frontend demo:

```bash
npm install
```

## Running the Demo

Start the development server:

```bash
npm run dev
```

The application will be available at [http://localhost:1234](http://localhost:1234)

## Updating the SDL

1. Install the Grafbase Postgres CLI extension:

```bash
curl -fsSL https://raw.githubusercontent.com/grafbase/extensions/refs/heads/main/cli/postgres/install.sh | bash
```

2. Create the products SDL:

```bash
grafbase postgres \
    --database-url postgres://postgres:grafbase@localhost:5432/products \
    introspect \
    --config grafbase/postgres-products.toml > grafbase/products.graphql
```

3. Create the inventory SDL:

```bash
grafbase postgres \
    --database-url postgres://postgres:grafbase@localhost:5432/inventory \
    introspect \
    --config grafbase/postgres-inventory.toml > grafbase/inventory.graphql
```

## Understanding Relay Pagination

This demo implements the Relay Connections specification for pagination, which uses:

- **Cursors**: Opaque markers that point to specific items in a collection
- **Connection**: Contains edges and pageInfo for traversing data
- **Edges**: Contains a node (the actual data) and a cursor
- **PageInfo**: Contains information about the current page like `hasNextPage` and `hasPreviousPage`

The UI demonstrates how to implement:

- Forward pagination (`first` + `after`)
- Backward pagination (`last` + `before`)
- Displaying pagination info

## Technologies Used

- Vanilla JavaScript (no framework to keep it simple)
- Parcel for bundling
- GraphQL for data fetching
- CSS for styling

## API Structure

The application connects to three main GraphQL queries from the Grafbase Postgres extension:

- `productsProducts`: Lists products with pagination
- `productsProduct.variants`: Lists variants for a specific product with pagination
- `inventoryInventories`: Lists inventory items for a specific variant with pagination
