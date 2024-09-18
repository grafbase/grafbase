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
#[graphql(
    field(name = "id", ty = "String"),
    field(name = "name", ty = "String", external = true)
    field(name = "qualifiedName", ty = "String", requires = "... on HomeingPigeon { nickname } ...on DeliveryCompany { companyType }")
)]
enum ShippingService {
    HomingPigeon(HomingPigeon),
    DeliveryCompany(DeliveryCompany),
}

struct Query;

#[Object]
impl Query {
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
