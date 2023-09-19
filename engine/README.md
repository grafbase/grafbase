<p align="center">
  <a href="https://grafbase.com">
    <img src="https://grafbase.com/images/other/grafbase-logo-circle.png" height="96">
    <h3 align="center">Engine 🏎️</h3>
  </a>
</p>

<p align="center">
  This workspace houses the Grafbase Engine, the core of the Grafbase platform and user generated APIs.
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

| Crate                        |                                  Description                                  |
| ---------------------------- | :---------------------------------------------------------------------------: |
| crates/async-runtime         |            A wrapper crate for various async runtime functionality            |
| crates/common-types          |              Various type definitions for the Grafbase platform               |
| crates/dataloader            |               A GraphQL dataloader implementation for Grafbase                |
| crates/dynamodb              | An implementation of the built-in Grafbase database using DynamoDB and SQLite |
| crates/dynamodb-utils        |                    Various utilities for use with DynamoDB                    |
| crates/engine                |                   A dynamic GraphQL engine written in Rust                    |
| crates/gateway-adapter       |                      A temporary adapter for the gateway                      |
| crates/gateway-adapter-local |                 Local gateway execution engine implementation                 |
| crates/graph-entities        |          Various types for use with GraphQL on the Grafbase platform          |
| crates/graphql-extensions    |                            Extensions for `engine`                            |
| crates/integration-tests     |                               Integration tests                               |
| crates/log                   |                Logging facilities for various Grafbase crates                 |
| crates/parser-graphql        |        A GraphQL schema parser for upstream APIs connected to Grafbase        |
| crates/parser-openapi        |              An OpenAPI schema parser for the Grafbase platform               |
| crates/parser-postgresql     |             Grafbase schema introspection for PostgreSQL database             |
| crates/parser-sdl            |    A parser that transforms GraphQL SDL into the Grafbase registry format     |
| crates/postgresql-types      |                     Shared types for PostgreSQL connector                     |
| crates/runtime               |         An abstraction over the various Grafbase runtime environments         |
| crates/runtime-local         |          An abstraction over the Grafbase local runtime environment           |
| crates/search-protocol       |          Types related to the Grafbase platform search functionality          |
| crates/worker-env            |    A utility crate to extend `worker::Env` with additional functionality.     |
