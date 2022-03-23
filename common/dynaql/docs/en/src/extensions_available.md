# Extensions available

There are a lot of available extensions in the `async-graphql` to empower your GraphQL Server, some of these documentations are documented here.

## Analyzer

_Available in the repository_

The `analyzer` extension will output a field containing `complexity` and `depth` in the response extension field of each query.

## Apollo Persisted Queries

_Available in the repository_

To improve network performance for large queries, you can enable this Persisted Queries extension. With this extension enabled, each unique query is associated to a unique identifier, so clients can send this identifier instead of the corresponding query string to reduce requests sizes.

This extension doesn't force you to use some cache strategy, you can choose the caching strategy you want, you'll just have to implement the `CacheStorage` trait:

```rust
#[async_trait::async_trait]
pub trait CacheStorage: Send + Sync + Clone + 'static {
    /// Load the query by `key`.
    async fn get(&self, key: String) -> Option<String>;
    /// Save the query by `key`.
    async fn set(&self, key: String, query: String);
}
```

References: [Apollo doc - Persisted Queries](https://www.apollographql.com/docs/react/api/link/persisted-queries/)

## Apollo Tracing

_Available in the repository_

Apollo Tracing is an extension which includes analytics data for your queries. This extension works to follow the old and now deprecated [Apollo Tracing Spec](https://github.com/apollographql/apollo-tracing). If you want to check the newer Apollo Reporting Protocol, it's implemented by [async-graphql Apollo studio extension](https://github.com/async-graphql/async_graphql_apollo_studio_extension) for Apollo Studio.

## Apollo Studio

_Available at [async-graphql/async_graphql_apollo_studio_extension](https://github.com/async-graphql/async_graphql_apollo_studio_extension)_

Apollo Studio is a cloud platform that helps you build, validate, and secure your organization's graph (description from the official documentation). It's a service allowing you to monitor & work with your team around your GraphQL Schema. `async-graphql` provides an extension implementing the official [Apollo Specification](https://www.apollographql.com/docs/studio/setup-analytics/#third-party-support) available at [async-graphql-extension-apollo-tracing](https://github.com/async-graphql/async_graphql_apollo_studio_extension) and [Crates.io](https://crates.io/crates/async-graphql-extension-apollo-tracing).

## Logger

_Available in the repository_

Logger is a simple extension allowing you to add some logging feature to `async-graphql`. It's also a good example to learn how to create your own extension.

## OpenTelemetry

_Available in the repository_

OpenTelemetry is an extension providing an integration with the [opentelemetry crate](https://crates.io/crates/opentelemetry) to allow your application to capture distributed traces and metrics from `async-grraphql`.

## Tracing

_Available in the repository_

Tracing is a simple extension allowing you to add some tracing feature to `async-graphql`. A little like the `Logger` extension.
