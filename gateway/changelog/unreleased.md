## Improvements

- The opt-in MCP server now takes HTTP headers into account when executing requests. For example, you can now configure your MCP client to send the `Authorization` header with a token as would be expected by the gateway, and the MCP server will pass on the header, allowing it to execute authenticated requests.
