use async_graphql::{
    ComplexObject, Context, EmptyMutation, EmptySubscription, FieldResult, Interface, Object, Schema, SimpleObject, ID,
};

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
        let shipping_services = vec![
            ShippingModality::DeliveryCompany(DeliveryCompany {
                id: "1".into(),
                name: "this subgraph does not know the name".to_string(),
                company_type: "should never be reached".to_string(),
            }),
            ShippingModality::HomingPigeon(HomingPigeon {
                id: "0".into(),
                name: "this subgraph does not know the name".to_string(),
                nickname: "should never be reached".to_string(),
            }),
        ];
        Schema::build(Query, EmptyMutation, EmptySubscription)
            .data(shipping_services)
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
#[graphql(complex)]
struct HomingPigeon {
    id: String,
    #[graphql(external)]
    name: String,
    #[graphql(external)]
    nickname: String,
}

#[derive(Clone, SimpleObject)]
#[graphql(complex)]
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
    field(name = "qualified_name", ty = "String",)
)]
enum ShippingModality {
    HomingPigeon(HomingPigeon),
    DeliveryCompany(DeliveryCompany),
}

#[ComplexObject]
impl HomingPigeon {
    #[graphql(requires = "nickname")]
    async fn qualified_name(&self, _ctx: &Context<'_>) -> FieldResult<String> {
        Ok(format!("{} a.k.a. {}", self.name, &self.nickname))
    }
}

#[ComplexObject]
impl DeliveryCompany {
    #[graphql(requires = "companyType")]
    async fn qualified_name(&self, _ctx: &Context<'_>) -> FieldResult<String> {
        Ok(format!("{} {}", self.name, &self.company_type))
    }
}

#[derive(SimpleObject)]
#[graphql(complex)]
struct ShippingOptions {
    modalities: Vec<ShippingModality>,
    default_delivery_company: DeliveryCompany,
    #[graphql(provides = "... on BusinessAccount { email } ... on User { reviewCount }")]
    seller: Account,
}

#[ComplexObject]
impl ShippingOptions {
    // #[graphql(requires = "modalities { ... on HomingPigeon { nickname } ...on DeliveryCompany { companyType } }")]
    async fn summary(&self) -> String {
        format!(
            "Shipping options: {}",
            self.modalities
                .iter()
                .map(|m| match m {
                    ShippingModality::HomingPigeon(p) => format!("{} a.k.a. {}", p.name, p.nickname),
                    ShippingModality::DeliveryCompany(c) => format!("{} {}", c.name, c.company_type),
                })
                .collect::<Vec<String>>()
                .join(", ")
        )
    }

    #[graphql(requires = "defaultDeliveryCompany { companyType }")]
    async fn default_company_summary(&self) -> String {
        format!(
            "Default company: {} {}",
            self.default_delivery_company.name, self.default_delivery_company.company_type
        )
    }
}

#[derive(SimpleObject, Clone)]
#[graphql(unresolvable)]
struct BusinessAccount {
    id: ID,
    #[graphql(external)]
    email: String,
    #[graphql(shareable)]
    joined_timestamp: u64,
}

#[derive(SimpleObject, Clone)]
#[graphql(unresolvable)]
struct User {
    id: ID,
    #[graphql(external)]
    username: String,
    #[graphql(external)]
    review_count: u64,
    #[graphql(shareable)]
    joined_timestamp: u64,
}

#[derive(Clone, async_graphql::Interface)]
#[graphql(field(name = "id", ty = "&ID"), field(name = "joined_timestamp", ty = "&u64"))]
enum Account {
    User(User),
    BusinessAccount(BusinessAccount),
}

struct Query;

#[Object]
impl Query {
    #[graphql(entity)]
    async fn find_homing_pidgen_by_id(
        &self,
        ctx: &Context<'_>,
        #[graphql(key)] id: String,
        nickname: Option<String>,
        name: Option<String>,
    ) -> HomingPigeon {
        let mut pigeon = ctx
            .data_unchecked::<Vec<ShippingModality>>()
            .iter()
            .find_map(|s| match s {
                ShippingModality::HomingPigeon(p) if p.id == id => Some(p.clone()),
                _ => None,
            })
            .unwrap();

        if let Some(name) = name {
            pigeon.name = name;
        }

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
        name: Option<String>,
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

        if let Some(name) = name {
            company.name = name;
        }

        if let Some(company_type) = company_type {
            company.company_type = company_type;
        }

        company
    }

    async fn shipping_options(&self, ctx: &Context<'_>) -> ShippingOptions {
        let modalities = ctx.data_unchecked::<Vec<ShippingModality>>().clone();
        let default_shipping_modality = modalities[0].clone();
        ShippingOptions {
            modalities,
            default_delivery_company: match default_shipping_modality {
                ShippingModality::DeliveryCompany(c) => c,
                _ => panic!("default shipping modality should be a delivery company"),
            },
            seller: Account::BusinessAccount(BusinessAccount {
                id: "ba_2".into(),
                email: "email@from-shipping-subgraph.net".to_owned(),
                joined_timestamp: 1234567890,
            }),
        }
    }
}
