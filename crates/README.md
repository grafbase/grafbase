<p align="center">
  <a href="https://grafbase.com">
    <img src="https://grafbase.com/images/other/grafbase-logo-circle.png" height="96">
    <h3 align="center">Grafbase crates</h3>
  </a>
</p>

<p align="center">
  This workspace houses the library crates used to build the Grafbase Platform, Gateway and CLi.
</p>

<p align="center">
  <a href="https://grafbase.com/docs/quickstart/get-started"><strong>Quickstart</strong></a> ·
  <a href="/examples"><strong>Examples</strong></a> ·
  <a href="/templates"><strong>Templates</strong></a> ·
  <a href="https://grafbase.com/docs"><strong>Docs</strong></a> ·
  <a href="https://grafbase.com/cli"><strong>CLI</strong></a> ·
  <a href="https://grafbase.com/community"><strong>Community</strong></a> ·
  <a href="https://grafbase.com/changelog"><strong>Changelog</strong></a>
</p>

<br/>

## Structure

| Crate                                                     | Description                                                         |
| --------------------------------------------------------- | ------------------------------------------------------------------- |
| [`common-types`](crates/common-types)                     | Various type definitions for the Grafbase platform                  |
| [`graphql-composition`](crates/composition)               | An implementation of GraphQL federated schema composition           |
| [`engine-config-builder`](crates/engine-config-builder)   | Engine configuration builder                                        |
| [`engine-v2`](crates/engine-v2)                           | A GraphQL federation engine                                         |
| [`federated-graph`](crates/federated-graph)               | A serializable federated GraphQL graph representation               |
| [`federation-audit-tests`](crates/federation-audit-tests) | Tests for federation auditing                                       |
| [`graphql-mocks`](crates/graphql-mocks)                   | GraphQL mocking utilities                                           |
| [`graphql-schema-diff`](crates/graphql-schema-diff)       | Semantic diffing for GraphQL schemas                                |
| [`integration-tests`](crates/integration-tests)           | Integration test suite                                              |
| [`operation-checks`](crates/operation-checks)             | GraphQL federation operation checks library                         |
| [`operation-normalizer`](crates/operation-normalizer)     | GraphQL operation normalizer                                        |
| [`runtime`](crates/runtime)                               | Runtime interfaces                                                  |
| [`runtime-local`](crates/runtime-local)                   | Runtime interface implementations for local execution               |
| [`runtime-noop`](crates/runtime-noop)                     | No-op runtime implementation                                        |
| [`serde-dynamic-string`](crates/serde-dynamic-string)     | Env var injection in Serde strings                                  |
| [`telemetry`](crates/telemetry)                           | Gateway telemetry utilities                                         |
| [`validation`](crates/validation)                         | A spec-compliant implementation of GraphQL SDL schema validation    |
| [`wasi-component-loader`](crates/wasi-component-loader)   | The Gateway hooks WASI runtime                                      |
| [`wrapping`](crates/wrapping)                             | Compact representation for GraphQL list and required wrapping types |

## Development

See [`DEVELOPMENT.md`](DEVELOPMENT.md)
