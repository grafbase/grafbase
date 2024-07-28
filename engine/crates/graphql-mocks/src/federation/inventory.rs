#![allow(clippy::duplicated_attributes)] // graphql false positive

use async_graphql::{
    ComplexObject, Context, EmptyMutation, EmptySubscription, Interface, Object, Schema, SimpleObject,
};

pub struct FederatedInventorySchema;

impl crate::Subgraph for FederatedInventorySchema {
    fn name(&self) -> String {
        "inventory".to_string()
    }
    async fn start(self) -> crate::MockGraphQlServer {
        crate::MockGraphQlServer::new(self).await
    }
}

impl FederatedInventorySchema {
    fn schema() -> Schema<Query, EmptyMutation, EmptySubscription> {
        let shiping_services = vec![
            ShippingService::DeliveryCompany(DeliveryCompany {
                id: "1".into(),
                name: "Planet Express".to_string(),
            }),
            ShippingService::HomingPigeon(HomingPigeon {
                id: "0".into(),
                name: "Cher Ami".to_string(),
            }),
        ];
        Schema::build(Query, EmptyMutation, EmptySubscription)
            .data(shiping_services)
            .enable_federation()
            .finish()
    }
}

#[async_trait::async_trait]
impl super::super::Schema for FederatedInventorySchema {
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

    #[graphql(requires = "weight(unit: KILOGRAM)")]
    async fn available_shipping_service(&self, ctx: &Context<'_>) -> Vec<ShippingService> {
        if self.weight <= 0.100 {
            ctx.data_unchecked::<Vec<ShippingService>>().clone()
        } else {
            ctx.data_unchecked::<Vec<ShippingService>>()
                .iter()
                .filter(|s| s.is_company())
                .cloned()
                .collect()
        }
    }

    #[graphql(external)]
    async fn weight(&self, _unit: WeightUnit) -> f64 {
        0.0
    }
}

#[derive(Clone, SimpleObject)]
struct HomingPigeon {
    id: String,
    name: String,
}

#[derive(Clone, SimpleObject)]
struct DeliveryCompany {
    id: String,
    name: String,
}

#[derive(Clone, Interface)]
#[graphql(field(name = "id", ty = "String"), field(name = "name", ty = "String"))]
enum ShippingService {
    HomingPigeon(HomingPigeon),
    DeliveryCompany(DeliveryCompany),
}

impl ShippingService {
    fn is_company(&self) -> bool {
        matches!(self, ShippingService::DeliveryCompany(_))
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

    #[graphql(entity)]
    async fn find_homing_pidgen_by_id(&self, ctx: &Context<'_>, id: String) -> HomingPigeon {
        ctx.data_unchecked::<Vec<ShippingService>>()
            .iter()
            .find_map(|s| match s {
                ShippingService::HomingPigeon(p) if p.id == id => Some(p.clone()),
                _ => None,
            })
            .unwrap()
    }

    #[graphql(entity)]
    async fn find_delivery_company_by_id(&self, ctx: &Context<'_>, id: String) -> DeliveryCompany {
        ctx.data_unchecked::<Vec<ShippingService>>()
            .iter()
            .find_map(|s| match s {
                ShippingService::DeliveryCompany(c) if c.id == id => Some(c.clone()),
                _ => None,
            })
            .unwrap()
    }

    #[graphql(entity)]
    async fn find_shipping_service_by_id(&self, ctx: &Context<'_>, id: String) -> ShippingService {
        ctx.data_unchecked::<Vec<ShippingService>>()
            .iter()
            .find(|s| match s {
                ShippingService::HomingPigeon(p) => p.id == id,
                ShippingService::DeliveryCompany(c) => c.id == id,
            })
            .cloned()
            .unwrap()
    }
}
