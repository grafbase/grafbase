// See https://github.com/async-graphql/examples
use async_graphql::{ComplexObject, EmptyMutation, EmptySubscription, Object, Schema, SimpleObject, ID};

pub struct FederatedAccountsSchema;

impl crate::Subgraph for FederatedAccountsSchema {
    fn name(&self) -> String {
        "accounts".to_string()
    }
    async fn start(self) -> crate::MockGraphQlServer {
        crate::MockGraphQlServer::new(self).await
    }
}

impl FederatedAccountsSchema {
    fn schema() -> Schema<Query, EmptyMutation, EmptySubscription> {
        Schema::build(Query, EmptyMutation, EmptySubscription)
            .enable_federation()
            .finish()
    }
}

#[async_trait::async_trait]
impl super::super::Schema for FederatedAccountsSchema {
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

#[derive(SimpleObject, Clone)]
struct BusinessAccount {
    id: ID,
    business_name: String,
    #[graphql(shareable)]
    email: String,
    joined_timestamp: u64,
}

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
struct User {
    id: ID,
    username: String,
    profile_picture: Option<Picture>,
    /// This used to be part of this subgraph, but is now being overridden from
    /// `reviews`
    review_count: u32,
    joined_timestamp: u64,
}

#[derive(Clone, async_graphql::Interface)]
#[graphql(field(name = "id", ty = "&ID"), field(name = "joined_timestamp", ty = "&u64"))]
enum Account {
    User(User),
    BusinessAccount(BusinessAccount),
}

impl User {
    fn me() -> User {
        User {
            id: "1234".into(),
            username: "Me".to_string(),
            profile_picture: Some(Picture {
                url: "http://localhost:8080/me.jpg".to_string(),
                width: 256,
                height: 256,
            }),
            review_count: 0,
            joined_timestamp: 1,
        }
    }
}

#[ComplexObject]
impl User {
    async fn cart(&self) -> Cart {
        Cart
    }
}

struct Cart;

#[Object]
impl Cart {
    async fn products(&self) -> Vec<Product> {
        vec![
            Product {
                name: "Fedora".to_string(),
            },
            Product {
                name: "Pink Jeans".to_string(),
            },
        ]
    }
}

#[derive(SimpleObject)]
#[graphql(unresolvable)]
struct Product {
    #[graphql(external)]
    name: String,
}

#[derive(SimpleObject, Clone)]
#[graphql(shareable)]
struct Picture {
    url: String,
    width: u32,
    height: u32,
}

struct Query;

#[Object]
impl Query {
    async fn me(&self) -> User {
        User::me()
    }

    #[graphql(entity)]
    async fn find_user_by_id(&self, id: ID) -> User {
        if id == "1234" {
            User::me()
        } else {
            let username = format!("User {}", id.as_str());
            User {
                id,
                username,
                profile_picture: None,
                review_count: 0,
                joined_timestamp: 1500,
            }
        }
    }

    #[graphql(entity)]
    async fn find_business_account_by_id(&self, id: ID) -> Option<BusinessAccount> {
        business_accounts().find(|account| account.id == id)
    }
}

fn business_accounts() -> impl Iterator<Item = BusinessAccount> {
    [
        BusinessAccount {
            id: "ba_1".into(),
            business_name: "Acme Corp".to_string(),
            email: "contact@acmecorp.com".to_string(),
            joined_timestamp: 1622548800,
        },
        BusinessAccount {
            id: "ba_2".into(),
            business_name: "Globex Corporation".to_string(),
            email: "info@globex.com".to_string(),
            joined_timestamp: 1625130800,
        },
        BusinessAccount {
            id: "ba_3".into(),
            business_name: "Initech".to_string(),
            email: "support@initech.com".to_string(),
            joined_timestamp: 1627819200,
        },
        BusinessAccount {
            id: "ba_4".into(),
            business_name: "Umbrella Corporation".to_string(),
            email: "admin@umbrella.com".to_string(),
            joined_timestamp: 1630411200,
        },
    ]
    .into_iter()
}
