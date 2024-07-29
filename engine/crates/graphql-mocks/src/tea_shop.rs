use async_graphql::{Context, EmptyMutation, EmptySubscription, Object, SimpleObject, Union};

use crate::Schema;

#[derive(Default)]
pub struct TeaShop {
    sdl: Option<String>,
}

impl TeaShop {
    pub fn with_sdl(sdl: &str) -> Self {
        TeaShop {
            sdl: Some(sdl.to_string()),
        }
    }
}

impl crate::Subgraph for TeaShop {
    fn name(&self) -> String {
        "tea-shop".to_string()
    }

    async fn start(self) -> crate::MockGraphQlServer {
        let schema = async_graphql::Schema::build(Query, EmptyMutation, EmptySubscription)
            .enable_federation()
            .enable_subscription_in_federation()
            .data(Data::default())
            .finish();
        if let Some(sdl) = self.sdl {
            crate::MockGraphQlServer::new(schema.with_sdl(&sdl)).await
        } else {
            crate::MockGraphQlServer::new(schema).await
        }
    }
}

struct Data {
    users: Vec<User>,
    teas: Vec<Tea>,
}

impl Default for Data {
    fn default() -> Self {
        Data {
            users: vec![
                User {
                    id: 0,
                    name: "Alice".to_string(),
                    address: Address {
                        street: "1234 Elm St".to_string(),
                    },
                    favorite_tea_id: Some(0),
                    orders: vec![Order { tea_id: 0, amount: 7 }, Order { tea_id: 2, amount: 2 }],
                },
                User {
                    id: 1,
                    name: "Bob".to_string(),
                    address: Address {
                        street: "5678 Oak St".to_string(),
                    },
                    favorite_tea_id: Some(1),
                    orders: vec![Order { tea_id: 1, amount: 3 }],
                },
                User {
                    id: 2,
                    name: "Charlie".to_string(),
                    address: Address {
                        street: "91011 Pine St".to_string(),
                    },
                    favorite_tea_id: Some(3),
                    orders: vec![
                        Order { tea_id: 3, amount: 3 },
                        Order { tea_id: 5, amount: 2 },
                        Order { tea_id: 6, amount: 1 },
                    ],
                },
                User {
                    id: 3,
                    name: "David".to_string(),
                    address: Address {
                        street: "121314 Maple St".to_string(),
                    },
                    favorite_tea_id: Some(4),
                    orders: vec![
                        Order { tea_id: 0, amount: 2 },
                        Order { tea_id: 1, amount: 1 },
                        Order { tea_id: 2, amount: 1 },
                        Order { tea_id: 4, amount: 5 },
                    ],
                },
            ],
            teas: vec![
                Tea {
                    id: 0,
                    name: "Earl Grey".to_string(),
                },
                Tea {
                    id: 1,
                    name: "Darjeeling".to_string(),
                },
                Tea {
                    id: 2,
                    name: "Assam".to_string(),
                },
                Tea {
                    id: 3,
                    name: "Ceylon".to_string(),
                },
                Tea {
                    id: 4,
                    name: "Matcha".to_string(),
                },
                Tea {
                    id: 5,
                    name: "Sencha".to_string(),
                },
                Tea {
                    id: 6,
                    name: "Gyokuro".to_string(),
                },
            ],
        }
    }
}

#[derive(Default)]
pub struct Query;

#[derive(SimpleObject, Clone)]
struct Tea {
    id: usize,
    name: String,
}

#[derive(Clone)]
struct User {
    id: usize,
    name: String,
    address: Address,
    favorite_tea_id: Option<usize>,
    orders: Vec<Order>,
}

#[Object]
impl User {
    async fn id(&self) -> usize {
        self.id
    }
    async fn name(&self) -> &str {
        &self.name
    }
    async fn address(&self) -> &Address {
        &self.address
    }
    async fn favorite_tea(&self, ctx: &Context<'_>) -> Option<Tea> {
        self.favorite_tea_id
            .map(|id| ctx.data_unchecked::<Data>().teas[id].clone())
    }
    async fn orders(&self) -> &Vec<Order> {
        &self.orders
    }
}

#[derive(Clone)]
struct Order {
    tea_id: usize,
    amount: usize,
}

#[Object]
impl Order {
    async fn tea(&self, ctx: &Context<'_>) -> Tea {
        let data = ctx.data_unchecked::<Data>();
        data.teas[self.tea_id].clone()
    }

    async fn amount(&self) -> usize {
        self.amount
    }
}

#[derive(SimpleObject, Clone)]
struct Address {
    street: String,
}

#[derive(Union)]
enum Node {
    Tea(Tea),
    User(User),
}

#[Object]
impl Query {
    async fn node(&self, ctx: &Context<'_>, id: String) -> Option<Node> {
        let data = ctx.data_unchecked::<Data>();
        let [ty, id] = id.split('#').collect::<Vec<_>>()[..] else {
            return None;
        };
        let id: usize = id.parse().ok()?;
        match ty {
            "Tea" => data.teas.get(id).map(|tea| Node::Tea(tea.clone())),
            "User" => data.users.get(id).map(|user| Node::User(user.clone())),
            _ => None,
        }
    }

    async fn user(&self, ctx: &Context<'_>, id: usize) -> Option<User> {
        ctx.data_unchecked::<Data>().users.get(id).cloned()
    }

    async fn users(&self, ctx: &Context<'_>) -> Vec<User> {
        ctx.data_unchecked::<Data>().users.clone()
    }
}
