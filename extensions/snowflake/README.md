# Snowflake Extension

This is a Snowflake database connector extension for the Grafbase Gateway.

** Work in progress. **

## Installing

Build this extension manually and copy the artifacts to a location where the gateway can find them.

```bash
grafbase extension build
```

The `build` directory contains the resulting wasm component and manifest file.

```bash
build/
├── extension.wasm
└── manifest.json
```

## Usage

Configure a Snowflake connection in your schema:

```graphql
extend schema
  @snowflakeConnection(
    name: "default",
    account: "your-account-identifier",
    username: "your-username",
    password: "your-password",
    database: "your-database",
    schema: "your-schema",
    warehouse: "your-warehouse", # Optional
    role: "your-role" # Optional
  )

type Query {
  users: [User!]! @snowflake(
    connection: "default",
    query: "SELECT * FROM USERS"
  )
  
  userById(id: Int!): User @snowflake(
    connection: "default",
    query: "SELECT * FROM USERS WHERE ID = ?",
    params: ["$id"]
  )
}

type User {
  id: Int!
  name: String!
  email: String!
  created_at: String!
}
``` 