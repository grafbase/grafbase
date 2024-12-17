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
| [`common-types`](common-types)                     | Various type definitions for the Grafbase platform                  |
| [`graphql-composition`](graphql-composition)               | An implementation of GraphQL federated schema composition           |
| [`engine-config-builder`](engine-config-builder)   | Engine configuration builder                                        |
| [`engine`](engine)                           | A GraphQL federation engine                                         |
| [`federated-graph`](graphql-federated-graph)               | A serializable federated GraphQL graph representation               |
| [`federation-audit-tests`](federation-audit-tests) | Tests for federation auditing                                       |
| [`graphql-mocks`](graphql-mocks)                   | GraphQL mocking utilities                                           |
| [`graphql-schema-diff`](graphql-schema-diff)       | Semantic diffing for GraphQL schemas                                |
| [`integration-tests`](integration-tests)           | Integration test suite                                              |
| [`operation-checks`](operation-checks)             | GraphQL federation operation checks library                         |
| [`operation-normalizer`](operation-normalizer)     | GraphQL operation normalizer                                        |
| [`runtime`](runtime)                               | Runtime interfaces                                                  |
| [`runtime-local`](runtime-local)                   | Runtime interface implementations for local execution               |
| [`runtime-noop`](runtime-noop)                     | No-op runtime implementation                                        |
| [`serde-dynamic-string`](serde-dynamic-string)     | Env var injection in Serde strings                                  |
| [`telemetry`](telemetry)                           | Gateway telemetry utilities                                         |
| [`validation`](graphql-validation)                         | A spec-compliant implementation of GraphQL SDL schema validation    |
| [`wasi-component-loader`](wasi-component-loader)   | The Gateway hooks WASI runtime                                      |
| [`wrapping`](graphql-wrapping-types)                             | Compact representation for GraphQL list and required wrapping types |

## Development

See [`DEVELOPMENT.md`](DEVELOPMENT.md)
