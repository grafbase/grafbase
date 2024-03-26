use async_graphql::{ComplexObject, EmptyMutation, EmptySubscription, Object, Schema, SimpleObject};

pub struct FakeFederationInventorySchema;

impl FakeFederationInventorySchema {
    fn schema() -> Schema<Query, EmptyMutation, EmptySubscription> {
        Schema::build(Query, EmptyMutation, EmptySubscription)
            .enable_federation()
            .finish()
    }
}

#[async_trait::async_trait]
impl super::super::Schema for FakeFederationInventorySchema {
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
#[graphql(complex)]
struct Product {
    upc: String,
    #[graphql(skip)]
    weight: f64,
}

#[derive(async_graphql::Enum, Clone, Copy, PartialEq, Eq)]
pub enum WeightUnit {
    Kilogram,
    Gram,
}

#[ComplexObject]
impl Product {
    #[graphql(requires = "weight(unit: KILOGRAM)")]
    async fn shipping_estimate(&self) -> u64 {
        if self.weight > 0.300 {
            3
        } else {
            1
        }
    }

    #[graphql(external)]
    async fn weight(&self, _unit: WeightUnit) -> f64 {
        0.0
    }
}

struct Query;

#[Object]
impl Query {
    #[graphql(entity)]
    async fn find_product_by_upc(&self, #[graphql(key)] upc: String, weight: Option<f64>) -> Product {
        Product {
            upc,
            weight: weight.unwrap_or(0.0),
        }
    }
}
