# Grafbase MCP server

This crate contains the implementation of the Model Context Protocol in Grafbase Gateway. You can find details on how to use it in the [docs page](https://grafbase.com/docs/features/mcp).

## Getting started

The Grafbase MCP server can be started with the Grafbase CLI by running:

```bash
npx grafbase mcp <url>
```

Where `<url>` is the URL of your GraphQL API.

The MCP server listens to requests at `http://127.0.0.1:5000/mcp` by default. To add it to Cursor, create a .cursor/mcp.json file in your project with the following:

```json
{
  "mcpServers": {
    "my-graphql-api": {
      "url": "http://127.0.0.1:5000/mcp"
    }
  }
}
```

## Setting up MCP in the Grafbase Gateway

The Grafbase Gateway can be configured to expose a MCP endpoint with the following grafbase.toml configuration:

```toml
[mcp]
enabled = true # defaults to false
# Path at which to expose the MCP service
path = "/mcp"
# Whether mutations can be executed
execute_mutations = false
```

Also see the [integration tests](https://github.com/grafbase/grafbase/tree/main/crates/integration-tests/tests/gateway/mcp).
