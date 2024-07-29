use async_graphql::{Any, Name, Object, SimpleObject, Value};

#[async_graphql::TypeDirective(
    name = "authorized",
    location = "FieldDefinition",
    location = "Object",
    composable = "https://custom.spec.dev/extension/v1.0"
)]
fn authorized(
    arguments: Option<String>,
    fields: Option<String>,
    node: Option<String>,
    metadata: Option<Any>,
) {
}

#[derive(Clone, Copy, SimpleObject)]
pub struct User {
    id: u64,
    name: &'static str,
    // @authorized(fields: "id")
    #[graphql(
        directive = authorized::apply(None, Some("id".to_string()), None, None)
    )]
    address: Option<Address>,
}

#[derive(Clone, Copy, SimpleObject)]
pub struct Transaction {
    id: u64,
    timestamp: i64,
    amount: f64,
}

#[derive(Clone, Copy, SimpleObject)]
pub struct Address {
    street: &'static str,
}

impl Default for Query {
    fn default() -> Self {
        let users = vec![
            User {
                id: 1,
                name: "Alice",
                address: Some(Address {
                    street: "123 Folsom",
                }),
            },
            User {
                id: 2,
                name: "Bob",
                address: Some(Address {
                    street: "123 Castro",
                }),
            },
            User {
                id: 3,
                name: "Musti",
                address: Some(Address {
                    street: "123 Planet",
                }),
            },
            User {
                id: 4,
                name: "Naukio",
                address: Some(Address {
                    street: "123 Rocket",
                }),
            },
        ];

        Self { users }
    }
}

pub struct Query {
    users: Vec<User>,
}

#[Object]
impl Query {
    // @authorized(node: "id", metadata: { role: "admin "})
    #[graphql(
        directive = authorized::apply(None, None, Some("id".to_string()), Some(Any(Value::Object([(Name::new("role"), "admin".into())].into()))))
    )]
    async fn users(&self) -> Vec<Option<&User>> {
        self.users.iter().map(Some).collect()
    }

    // @authorized(arguments: "id")
    #[graphql(
        directive = authorized::apply(Some("id".to_string()), None, None, None)
    )]
    async fn user<'a>(&self, id: u64) -> Option<&User> {
        self.users.iter().find(|user| user.id == id)
    }
}
