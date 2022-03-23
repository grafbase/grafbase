# 可用的扩展列表

`async-graphql` 中有很多可用的扩展用于增强你的 GraphQL 服务器。

## Analyzer

_Available in the repository_

`Analyzer` 扩展将在每个响应的扩展中输出 `complexity` 和 `depth` 字段。

## Apollo Persisted Queries

_Available in the repository_

要提高大型查询的性能，你可以启用此扩展，每个查询语句都将与一个唯一 ID 相关联，因此客户端可以直接发送此 ID 查询以减少请求的大小。

这个扩展不会强迫你使用一些缓存策略，你可以选择你想要的缓存策略，你只需要实现 `CacheStorage` trait：

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

`Apollo Tracing` 扩展用于在响应中包含此查询分析数据。 此扩展程序遵循旧的且现已弃用的 [Apollo Tracing Spec](https://github.com/apollographql/apollo-tracing) 。
如果你想支持更新的 Apollo Reporting Protocol，推荐使用 [async-graphql Apollo studio extension](https://github.com/async-graphql/async_graphql_apollo_studio_extension) 。

## Apollo Studio

_Available at [async-graphql/async_graphql_apollo_studio_extension](https://github.com/async-graphql/async_graphql_apollo_studio_extension)_

`async-graphql` 提供了实现官方 [Apollo Specification](https://www.apollographql.com/docs/studio/setup-analytics/#third-party-support) 的扩展，位于 [async-graphql-extension- apollo-tracing](https://github.com/async-graphql/async_graphql_apollo_studio_extension) 和 [crates.io](https://crates.io/crates/async-graphql-extension-apollo-tracing) 。

## Logger

_Available in the repository_

`Logger` 是一个简单的扩展，允许你向 `async-graphql` 添加一些日志记录功能。这也是学习如何创建自己的扩展的一个很好的例子。

## OpenTelemetry

_Available in the repository_

`OpenTelemetry` 扩展提供 [opentelemetry crate](https://crates.io/crates/opentelemetry) 的集成，以允许你的应用程序从 `async-graphql` 捕获分布式跟踪和指标。

## Tracing

_Available in the repository_

`Tracing` 扩展提供 [tracing crate](https://crates.io/crates/tracing) 的集成，允许您向 `async-graphql` 添加一些跟踪功能，有点像`Logger` 扩展。
