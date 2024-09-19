use async_graphql::{Context, EmptyMutation, EmptySubscription, FieldResult, Interface, Object, Schema, SimpleObject};

pub struct FederatedShippingSchema;

impl crate::Subgraph for FederatedShippingSchema {
    fn name(&self) -> String {
        "shipping".to_string()
    }
    async fn start(self) -> crate::MockGraphQlServer {
        crate::MockGraphQlServer::new(self).await
    }
}

impl FederatedShippingSchema {
    fn schema() -> Schema<Query, EmptyMutation, EmptySubscription> {
        let shiping_services = vec![
            ShippingModality::DeliveryCompany(DeliveryCompany {
                id: "1".into(),
                name: "Planet Express".to_string(),
                company_type: "should never be reached".to_string(),
            }),
            ShippingModality::HomingPigeon(HomingPigeon {
                id: "0".into(),
                name: "Cher Ami".to_string(),
                nickname: "should never be reached".to_string(),
            }),
        ];
        Schema::build(Query, EmptyMutation, EmptySubscription)
            .data(shiping_services)
            .enable_federation()
            .finish()
    }
}

#[async_trait::async_trait]
impl super::super::Schema for FederatedShippingSchema {
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
    #[graphql(external)]
    name: String,
    #[graphql(external)]
    nickname: String,
}

#[derive(Clone, SimpleObject)]
struct DeliveryCompany {
    id: String,
    #[graphql(external)]
    name: String,
    #[graphql(external)]
    company_type: String,
}

#[derive(Clone, Interface)]
#[graphql(
    field(name = "id", ty = "String"),
    field(name = "name", ty = "String", external = true),
    field(
        name = "qualified_name",
        ty = "String",
        requires = "... on HomeingPigeon { nickname } ...on DeliveryCompany { companyType }"
    )
)]
enum ShippingModality {
    HomingPigeon(HomingPigeon),
    DeliveryCompany(DeliveryCompany),
}

impl HomingPigeon {
    async fn qualified_name(&self, _ctx: &Context<'_>) -> FieldResult<String> {
        Ok(format!("{} a.k.a. {}", self.name, &self.nickname))
    }
}

impl DeliveryCompany {
    async fn qualified_name(&self, _ctx: &Context<'_>) -> FieldResult<String> {
        Ok(format!("{} {}", self.name, &self.company_type))
    }
}

struct Query;

#[Object]
impl Query {
    #[graphql(entity)]
    async fn find_shipping_service_by_id(&self, ctx: &Context<'_>, id: String) -> ShippingModality {
        ctx.data_unchecked::<Vec<ShippingModality>>()
            .iter()
            .find(|s| match s {
                ShippingModality::HomingPigeon(p) => p.id == id,
                ShippingModality::DeliveryCompany(c) => c.id == id,
            })
            .cloned()
            .unwrap()
    }

    #[graphql(entity)]
    async fn find_homing_pidgen_by_id(
        &self,
        ctx: &Context<'_>,
        #[graphql(key)] id: String,
        nickname: Option<String>,
    ) -> HomingPigeon {
        let mut pigeon = ctx
            .data_unchecked::<Vec<ShippingModality>>()
            .iter()
            .find_map(|s| match s {
                ShippingModality::HomingPigeon(p) if p.id == id => Some(p.clone()),
                _ => None,
            })
            .unwrap();

        if let Some(nickname) = nickname {
            pigeon.nickname = nickname
        }

        pigeon
    }

    #[graphql(entity)]
    async fn find_delivery_company_by_id(
        &self,
        ctx: &Context<'_>,
        #[graphql(key)] id: String,
        company_type: Option<String>,
    ) -> DeliveryCompany {
        let mut company = ctx
            .data_unchecked::<Vec<ShippingModality>>()
            .iter()
            .find_map(|s| match s {
                ShippingModality::DeliveryCompany(c) if c.id == id => Some(c.clone()),
                _ => None,
            })
            .unwrap();

        if let Some(company_type) = company_type {
            company.company_type = company_type;
        }

        company
    }

    async fn shipping_modalities(&self, ctx: &Context<'_>) -> Vec<ShippingModality> {
        ctx.data_unchecked::<Vec<ShippingModality>>().clone()
    }
}
