// See https://github.com/async-graphql/examples
use async_graphql::{Context, EmptyMutation, EmptySubscription, Object, Schema, SimpleObject};

pub struct FakeFederationProductsSchema;

impl FakeFederationProductsSchema {
    fn schema() -> Schema<Query, EmptyMutation, EmptySubscription> {
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
        ];
        Schema::build(Query, EmptyMutation, EmptySubscription)
            .enable_federation()
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

    fn sdl(&self) -> String {
        Self::schema().sdl_with_options(async_graphql::SDLExportOptions::new())
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
}
