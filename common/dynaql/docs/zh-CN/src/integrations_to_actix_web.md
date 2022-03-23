# Actix-web

`Async-graphql-actix-web`提供了`GraphQLRequest`提取器用于提取`GraphQL`请求，和`GraphQLResponse`用于输出`GraphQL`响应。

`GraphQLSubscription`用于创建一个支持 Web Socket 订阅的 Actor。

## 请求例子

你需要把 Schema 传入`actix_web::App`作为全局数据。

```rust
async fn index(
    schema: web::Data<Schema>,
    request: GraphQLRequest,
) -> web::Json<GraphQLResponse> {
    web::Json(schema.execute(request.into_inner()).await.into())
}
```

## 订阅例子

```rust
async fn index_ws(
    schema: web::Data<Schema>,
    req: HttpRequest,
    payload: web::Payload,
) -> Result<HttpResponse> {
    GraphQLSubscription::new(Schema::clone(&*schema)).start(&req, payload)
}
```

## 更多例子

[https://github.com/async-graphql/examples/tree/master/actix-web](https://github.com/async-graphql/examples/tree/master/actix-web)
