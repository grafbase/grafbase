use async_graphql::{EmptyMutation, EmptySubscription, Object};

/// A schema that only uses String types.
///
/// This is used to make sure that we're not pruning built in scalars that aren't used
pub type AlmostEmptySchema = async_graphql::Schema<Query, EmptyMutation, EmptySubscription>;

#[derive(Default)]
pub struct Query;

#[Object]
impl Query {
    async fn string(&self, input: String) -> String {
        input
    }
}
