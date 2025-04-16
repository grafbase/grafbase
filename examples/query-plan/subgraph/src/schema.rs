use std::sync::atomic::AtomicUsize;

use async_graphql::{ComplexObject, Object, SimpleObject, ID};

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
        Some(self.clone())
    }
    async fn n1(&self) -> Option<Node> {
        Some(self.clone())
    }
    async fn n2(&self) -> Option<Node> {
        Some(self.clone())
    }
    async fn n3(&self) -> Option<Node> {
        Some(self.clone())
    }
    async fn n4(&self) -> Option<Node> {
        Some(self.clone())
    }
    async fn n5(&self) -> Option<Node> {
        Some(self.clone())
    }
    async fn n6(&self) -> Option<Node> {
        Some(self.clone())
    }
    async fn n7(&self) -> Option<Node> {
        Some(self.clone())
    }
    async fn n8(&self) -> Option<Node> {
        Some(self.clone())
    }
    async fn n9(&self) -> Option<Node> {
        Some(self.clone())
    }
}

impl Default for Query {
    fn default() -> Self {
        Self {
            node: Node {
                id0: "0".to_string(),
                id1: "1".to_string(),
                id2: "2".to_string(),
                id3: "3".to_string(),
                id4: "4".to_string(),
                id5: "5".to_string(),
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
            },
        }
    }
}

pub struct Query {
    node: Node,
}

#[Object]
impl Query {
    async fn node(&self) -> Option<Node> {
        sleep().await;
        Some(self.node.clone())
    }

    #[graphql(entity)]
    async fn find_node_by_id0(&self, id0: ID) -> Node {
        sleep().await;
        let _ = id0;
        self.node.clone()
    }

    #[graphql(entity)]
    async fn find_node_by_id1(&self, id1: ID) -> Node {
        sleep().await;
        let _ = id1;
        self.node.clone()
    }

    #[graphql(entity)]
    async fn find_node_by_id2(&self, id2: ID) -> Node {
        sleep().await;
        let _ = id2;
        self.node.clone()
    }

    #[graphql(entity)]
    async fn find_node_by_id3(&self, id3: ID) -> Node {
        sleep().await;
        let _ = id3;
        self.node.clone()
    }

    #[graphql(entity)]
    async fn find_node_by_id4(&self, id4: ID) -> Node {
        sleep().await;
        let _ = id4;
        self.node.clone()
    }

    #[graphql(entity)]
    async fn find_node_by_id5(&self, id5: ID) -> Node {
        sleep().await;
        let _ = id5;
        self.node.clone()
    }
}

static COUNT: AtomicUsize = AtomicUsize::new(0);

async fn sleep() {
    let current = COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    println!("executing {current}");
    if let Some(delay) = std::env::var("DELAY_MS").ok().and_then(|v| v.parse().ok()) {
        tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
    }
}
