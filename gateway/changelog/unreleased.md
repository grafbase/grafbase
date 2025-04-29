## Breaking changes

- The gateway will now try to expand environment variables everywhere in the configuration with the syntax `{{ env>VAR_NAME }}`.

## Fixes

- The gateway now accepts the `extension` directive on the `@join__type` federated directive. It is valid but previously caused a validation error on startup.
