## Improvements

- GraphQL schema validation is now run on every subgraph schema before composition in `grafbase dev` and `grafbase compose`. That makes for better error messages.

## Breaking changes

- The CLI will now try to expand environment variables everywhere in the `grafbase.toml` configuration with the syntax `{{ env>VAR_NAME }}`.
