use async_graphql::{Object, SimpleObject};

#[derive(Clone, Copy, SimpleObject)]
pub struct User {
    id: u64,
    name: &'static str,
    address: Address,
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
                address: Address { street: "123 Folsom" },
            },
            User {
                id: 2,
                name: "Bob",
                address: Address { street: "123 Castro" },
            },
            User {
                id: 3,
                name: "Musti",
                address: Address { street: "123 Planet" },
            },
            User {
                id: 4,
                name: "Naukio",
                address: Address { street: "123 Rocket" },
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
    async fn users(&self) -> &Vec<User> {
        &self.users
    }

    async fn user<'a>(&self, id: u64) -> Option<&User> {
        self.users.iter().find(|user| user.id == id)
    }
}
