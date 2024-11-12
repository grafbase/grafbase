// See https://github.com/async-graphql/examples
use async_graphql::{ComplexObject, Context, EmptyMutation, Object, Schema, SimpleObject};
use futures::Stream;

pub struct FederatedProductsSchema;

impl crate::Subgraph for FederatedProductsSchema {
    fn name(&self) -> String {
        "products".to_string()
    }
    async fn start(self) -> crate::MockGraphQlServer {
        crate::MockGraphQlServer::new(self).await
    }
}

impl FederatedProductsSchema {
    fn schema() -> Schema<Query, EmptyMutation, Subscription> {
        let products = vec![
            Product {
                upc: "top-1".to_string(),
                name: "Trilby".to_string(),
                price: 11,
                weight_grams: 100,
            },
            Product {
                upc: "top-2".to_string(),
                name: "Fedora".to_string(),
                price: 22,
                weight_grams: 200,
            },
            Product {
                upc: "top-3".to_string(),
                name: "Boater".to_string(),
                price: 33,
                weight_grams: 300,
            },
            Product {
                upc: "top-4".to_string(),
                name: "Jeans".to_string(),
                price: 44,
                weight_grams: 400,
            },
            Product {
                upc: "top-5".to_string(),
                name: "Pink Jeans".to_string(),
                price: 55,
                weight_grams: 500,
            },
        ];
        Schema::build(Query, EmptyMutation, Subscription)
            .enable_federation()
            .enable_subscription_in_federation()
            .data(products)
            .finish()
    }
}

#[async_trait::async_trait]
impl super::super::Schema for FederatedProductsSchema {
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

#[derive(async_graphql::Enum, Clone, Copy, PartialEq, Eq)]
pub enum WeightUnit {
    Kilogram,
    Gram,
}

#[derive(Clone, SimpleObject)]
#[graphql(complex)]
struct Product {
    upc: String,
    name: String,
    #[graphql(shareable)]
    price: i32,
    #[graphql(skip)]
    weight_grams: u64,
}

#[ComplexObject]
impl Product {
    async fn weight(&self, unit: WeightUnit) -> f64 {
        match unit {
            WeightUnit::Kilogram => (self.weight_grams as f64) / 1000.0,
            WeightUnit::Gram => self.weight_grams as f64,
        }
    }
}

struct Query;

#[Object]
impl Query {
    async fn top_products<'a>(&self, ctx: &'a Context<'_>) -> &'a Vec<Product> {
        ctx.data_unchecked::<Vec<Product>>()
    }

    async fn product<'a>(&self, ctx: &'a Context<'_>, upc: String) -> Option<&'a Product> {
        let products = ctx.data_unchecked::<Vec<Product>>();
        products.iter().find(|product| product.upc == upc)
    }

    #[graphql(entity)]
    async fn find_product_by_upc<'a>(&self, ctx: &'a Context<'_>, upc: String) -> Option<&'a Product> {
        let products = ctx.data_unchecked::<Vec<Product>>();
        products.iter().find(|product| product.upc == upc)
    }

    #[graphql(entity)]
    async fn find_product_by_name<'a>(&self, ctx: &'a Context<'_>, name: String) -> Option<&'a Product> {
        let products = ctx.data_unchecked::<Vec<Product>>();
        products.iter().find(|product| product.name == name)
    }
}

struct Subscription;

#[async_graphql::Subscription]
impl Subscription {
    async fn new_products(&self, ctx: &Context<'_>) -> impl Stream<Item = Product> {
        futures::stream::iter(
            ctx.data_unchecked::<Vec<Product>>()
                .iter()
                .filter(|product| product.upc == "top-4" || product.upc == "top-5")
                .cloned()
                .collect::<Vec<Product>>(),
        )
    }
}
