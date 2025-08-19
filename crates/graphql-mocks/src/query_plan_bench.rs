use async_graphql::{ComplexObject, EmptyMutation, EmptySubscription, ID, Object, SimpleObject};

pub struct QueryBenchSchema {
    schema: async_graphql::Schema<Query, EmptyMutation, EmptySubscription>,
}

impl crate::Subgraph for QueryBenchSchema {
    fn name(&self) -> String {
        "secure".to_string()
    }

    async fn start(self) -> crate::MockGraphQlServer {
        crate::MockGraphQlServer::new(self.schema).await
    }
}

impl Default for QueryBenchSchema {
    fn default() -> Self {
        QueryBenchSchema {
            schema: async_graphql::Schema::build(Query::default(), EmptyMutation, EmptySubscription).finish(),
        }
    }
}

#[derive(Clone, SimpleObject)]
#[graphql(complex)]
pub struct Node {
    id0: String,
    id1: String,
    id2: String,
    id3: String,
    id4: String,
    id5: String,
    f0: Option<String>,
    f1: Option<String>,
    f2: Option<String>,
    f3: Option<String>,
    f4: Option<String>,
    f5: Option<String>,
    f6: Option<String>,
    f7: Option<String>,
    f8: Option<String>,
    f9: Option<String>,
}

#[ComplexObject]
impl Node {
    async fn n0(&self) -> Option<Node> {
        Some(Self::new())
    }
    async fn n1(&self) -> Option<Node> {
        Some(Self::new())
    }
    async fn n2(&self) -> Option<Node> {
        Some(Self::new())
    }
    async fn n3(&self) -> Option<Node> {
        Some(Self::new())
    }
    async fn n4(&self) -> Option<Node> {
        Some(Self::new())
    }
    async fn n5(&self) -> Option<Node> {
        Some(Self::new())
    }
    async fn n6(&self) -> Option<Node> {
        Some(Self::new())
    }
    async fn n7(&self) -> Option<Node> {
        Some(Self::new())
    }
    async fn n8(&self) -> Option<Node> {
        Some(Self::new())
    }
    async fn n9(&self) -> Option<Node> {
        Some(Self::new())
    }
}

impl Node {
    pub fn new() -> Self {
        Node {
            id0: "id1".to_string(),
            id1: "id2".to_string(),
            id2: "id3".to_string(),
            id3: "id4".to_string(),
            id4: "id5".to_string(),
            id5: "id6".to_string(),
            f0: Some("f0".to_string()),
            f1: Some("f1".to_string()),
            f2: Some("f2".to_string()),
            f3: Some("f3".to_string()),
            f4: Some("f4".to_string()),
            f5: Some("f5".to_string()),
            f6: Some("f6".to_string()),
            f7: Some("f7".to_string()),
            f8: Some("f8".to_string()),
            f9: Some("f9".to_string()),
        }
    }
}

#[derive(Default)]
pub struct Query {}

#[Object]
impl Query {
    async fn node(&self) -> Option<Node> {
        Some(Node::new())
    }

    #[graphql(entity)]
    async fn find_node_by_id0(&self, id0: ID) -> Node {
        Node {
            id0: id0.to_string(),
            ..Node::new()
        }
    }

    #[graphql(entity)]
    async fn find_node_by_id1(&self, id1: ID) -> Node {
        Node {
            id1: id1.to_string(),
            ..Node::new()
        }
    }

    #[graphql(entity)]
    async fn find_node_by_id2(&self, id2: ID) -> Node {
        Node {
            id2: id2.to_string(),
            ..Node::new()
        }
    }

    #[graphql(entity)]
    async fn find_node_by_id3(&self, id3: ID) -> Node {
        Node {
            id3: id3.to_string(),
            ..Node::new()
        }
    }

    #[graphql(entity)]
    async fn find_node_by_id4(&self, id4: ID) -> Node {
        Node {
            id4: id4.to_string(),
            ..Node::new()
        }
    }

    #[graphql(entity)]
    async fn find_node_by_id5(&self, id5: ID) -> Node {
        Node {
            id5: id5.to_string(),
            ..Node::new()
        }
    }
}
