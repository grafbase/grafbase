// See https://github.com/async-graphql/examples
use async_graphql::{ComplexObject, EmptyMutation, EmptySubscription, Object, Schema, SimpleObject, ID};

pub struct FakeFederationAccountsSchema;

impl FakeFederationAccountsSchema {
    fn schema() -> Schema<Query, EmptyMutation, EmptySubscription> {
        Schema::build(Query, EmptyMutation, EmptySubscription)
            .enable_federation()
            .finish()
    }
}

#[async_trait::async_trait]
impl super::super::Schema for FakeFederationAccountsSchema {
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
struct User {
    id: ID,
    username: String,
    profile_picture: Option<Picture>,
    /// This used to be part of this subgraph, but is now being overridden from
    /// `reviews`
    review_count: u32,
    joined_timestamp: u64,
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

#[derive(SimpleObject)]
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
}
