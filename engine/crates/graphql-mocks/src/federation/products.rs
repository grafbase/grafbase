// See https://github.com/async-graphql/examples
use async_graphql::{Context, EmptyMutation, Object, Schema, SimpleObject};
use futures::Stream;

pub struct FakeFederationProductsSchema;

impl FakeFederationProductsSchema {
    fn schema() -> Schema<Query, EmptyMutation, Subscription> {
        let hats = vec![
            Product {
                upc: "top-1".to_string(),
                name: "Trilby".to_string(),
                price: 11,
            },
            Product {
                upc: "top-2".to_string(),
                name: "Fedora".to_string(),
                price: 22,
            },
            Product {
                upc: "top-3".to_string(),
                name: "Boater".to_string(),
                price: 33,
            },
            Product {
                upc: "top-4".to_string(),
                name: "Jeans".to_string(),
                price: 44,
            },
            Product {
                upc: "top-5".to_string(),
                name: "Pink Jeans".to_string(),
                price: 55,
            },
        ];
        Schema::build(Query, EmptyMutation, Subscription)
            .enable_federation()
            .enable_subscription_in_federation()
            .data(hats)
            .finish()
    }
}

#[async_trait::async_trait]
impl super::super::Schema for FakeFederationProductsSchema {
    async fn execute(
        &self,
        _headers: Vec<(String, String)>,
        request: async_graphql::Request,
    ) -> async_graphql::Response {
        Self::schema().execute(request).await
    }

    fn execute_stream(
        &self,
        request: async_graphql::Request,
    ) -> futures::stream::BoxStream<'static, async_graphql::Response> {
        Box::pin(Self::schema().execute_stream(request))
    }

    fn sdl(&self) -> String {
        Self::schema().sdl_with_options(async_graphql::SDLExportOptions::new().federation())
    }
}

#[derive(SimpleObject)]
struct Product {
    upc: String,
    name: String,
    #[graphql(shareable)]
    price: i32,
}

struct Query;

#[Object]
impl Query {
    async fn top_products<'a>(&self, ctx: &'a Context<'_>) -> &'a Vec<Product> {
        ctx.data_unchecked::<Vec<Product>>()
    }

    #[graphql(entity)]
    async fn find_product_by_upc<'a>(&self, ctx: &'a Context<'_>, upc: String) -> Option<&'a Product> {
        let hats = ctx.data_unchecked::<Vec<Product>>();
        hats.iter().find(|product| product.upc == upc)
    }

    #[graphql(entity)]
    async fn find_product_by_name<'a>(&self, ctx: &'a Context<'_>, name: String) -> Option<&'a Product> {
        let hats = ctx.data_unchecked::<Vec<Product>>();
        hats.iter().find(|product| product.name == name)
    }
}

struct Subscription;

#[async_graphql::Subscription]
impl Subscription {
    async fn new_products(&self) -> impl Stream<Item = Product> {
        futures::stream::iter([
            Product {
                upc: "top-4".to_string(),
                name: "Jeans".to_string(),
                price: 44,
            },
            Product {
                upc: "top-5".to_string(),
                name: "Pink Jeans".to_string(),
                price: 55,
            },
        ])
    }
}
