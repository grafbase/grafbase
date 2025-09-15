# MCP (Model Context Protocol) Crate

## Overview

This crate implements the Model Context Protocol (MCP) server for Grafbase Gateway, enabling LLMs like Claude/Cursor to interact with GraphQL APIs through a standardized interface.

## Key Components

### Main Module (`lib.rs`)

- Creates an MCP router that can be integrated into the Grafbase Gateway
- Supports two transport modes:
  - **StreamingHttp**: Direct HTTP-based streaming communication
  - **SSE (Server-Sent Events)**: Event-based communication
- Configuration controlled via `ModelControlProtocolConfig`

### Server Implementation (`server.rs`)

- `McpServer`: Core server that handles MCP protocol requests
- Registers three main tools for GraphQL interaction
- Uses `rmcp` crate for MCP protocol implementation
- Server capabilities include tool support

### Tools

The MCP server provides three GraphQL tools:

1. **SearchTool** (`tools/search/`)

   - Searches for relevant GraphQL fields using keyword matching
   - Uses Tantivy for full-text search indexing
   - Returns partial SDL with matching fields and relevance scores
   - Tokenizes queries for better matching (handles camelCase, snake_case, etc.)

2. **IntrospectTool** (`tools/introspect.rs`)

   - Provides complete GraphQL SDL for requested types
   - Essential for understanding schema structure
   - Returns type definitions with all fields and nested types
   - Marked as read-only operation

3. **ExecuteTool** (`tools/execute.rs`)
   - Executes GraphQL queries/mutations against the API
   - Accepts query string and variables
   - Can be configured to allow/disallow mutations
   - Returns query results in MCP format

### SDL Generation (`tools/sdl/`)

- `PartialSdl` trait for generating GraphQL SDL snippets
- Buffer-based SDL building for efficient string concatenation
- Handles complex GraphQL type definitions

## Dependencies

Key dependencies:

- `rmcp`: MCP protocol implementation
- `tantivy`: Full-text search for the search tool
- `engine`/`engine-schema`: GraphQL engine integration
- `axum`: Web framework for HTTP transport
- `tokio`: Async runtime

## Integration Points

- The crate watches for engine updates via `EngineWatcher`
- Integrates with gateway configuration system
- Works with existing GraphQL schema and operation execution

## Testing

Integration tests located at: `crates/integration-tests/tests/gateway/mcp`
