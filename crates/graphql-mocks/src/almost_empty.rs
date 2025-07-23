use async_graphql::{EmptyMutation, EmptySubscription, Object};

/// A schema that only uses String types.
///
/// This is used to make sure that we're not pruning built in scalars that aren't used
pub struct AlmostEmptySchema {
    schema: async_graphql::Schema<Query, EmptyMutation, EmptySubscription>,
}

impl crate::Subgraph for AlmostEmptySchema {
    fn name(&self) -> String {
        "almost-empty".to_string()
    }
    async fn start(self) -> crate::MockGraphQlServer {
        crate::MockGraphQlServer::new(self.schema).await
    }
}

impl Default for AlmostEmptySchema {
    fn default() -> Self {
        AlmostEmptySchema {
            schema: async_graphql::Schema::build(Query, EmptyMutation, EmptySubscription).finish(),
        }
    }
}

#[derive(Default)]
pub struct Query;

#[Object]
impl Query {
    async fn string(&self, input: String) -> String {
        input
    }
}
