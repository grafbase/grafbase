use std::collections::HashMap;

use async_graphql::{Context, Object, SimpleObject, TypeDirective};

#[TypeDirective(
    name = "authorized",
    location = "FieldDefinition",
    location = "Object",
    composable = "https://custom.spec.dev/extension/v1.0"
)]
fn authorized(arguments: Option<String>) {}

pub struct QueryRoot;

pub struct Users {
    users: HashMap<u64, User>,
}

impl Default for Users {
    fn default() -> Self {
        Self::new()
    }
}

impl Users {
    pub fn new() -> Self {
        let mut users = HashMap::new();

        users.insert(
            1,
            User {
                id: 1,
                name: "Alice",
                address: Address { street: "123 Folsom" },
                secret: Some(Secret {
                    id: 1,
                    social_security_number: "1234",
                }),
            },
        );

        users.insert(
            2,
            User {
                id: 2,
                name: "Bob",
                address: Address { street: "123 Castro" },
                secret: Some(Secret {
                    id: 2,
                    social_security_number: "456",
                }),
            },
        );

        users.insert(
            3,
            User {
                id: 3,
                name: "Musti",
                address: Address { street: "123 Planet" },
                secret: Some(Secret {
                    id: 3,
                    social_security_number: "789",
                }),
            },
        );

        users.insert(
            4,
            User {
                id: 4,
                name: "Naukio",
                address: Address { street: "123 Rocket" },
                secret: Some(Secret {
                    id: 4,
                    social_security_number: "999",
                }),
            },
        );

        Self { users }
    }
}

#[derive(Clone, Copy, SimpleObject)]
pub struct User {
    id: u64,
    name: &'static str,
    address: Address,
    secret: Option<Secret>,
}

#[derive(Clone, Copy, SimpleObject)]
pub struct Address {
    street: &'static str,
}

#[derive(Clone, Copy, SimpleObject)]
#[graphql(
    directive = authorized::apply(None)
)]
pub struct Secret {
    id: u64,
    social_security_number: &'static str,
}

#[Object]
impl QueryRoot {
    async fn get_user<'a>(&self, ctx: &Context<'a>, id: u64) -> Option<User> {
        let users: &Users = ctx.data().unwrap();
        users.users.get(&id).copied()
    }

    #[graphql(
        directive = authorized::apply(Some("id".to_string()))
    )]
    async fn get_secret<'a>(&self, ctx: &Context<'a>, id: u64) -> Option<Secret> {
        let users: &Users = ctx.data().unwrap();
        users.users.get(&id).and_then(|user| user.secret)
    }
}
