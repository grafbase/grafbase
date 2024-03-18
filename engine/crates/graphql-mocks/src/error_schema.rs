use async_graphql::{EmptyMutation, EmptySubscription, FieldResult, Object};

/// A schema that exposes a field with errors
pub type ErrorSchema = async_graphql::Schema<Query, EmptyMutation, EmptySubscription>;

#[derive(Default)]
pub struct Query;

#[Object]
impl Query {
    async fn broken_field(&self, error: String) -> FieldResult<String> {
        Err(error.into())
    }

    async fn broken_list(&self, error: String) -> FieldResult<Vec<String>> {
        Err(error.into())
    }

    async fn broken_object_list(&self, error: String) -> Option<Vec<Option<BrokenObject>>> {
        Some(vec![
            Some(BrokenObject { error: error.clone() }),
            Some(BrokenObject { error: error.clone() }),
        ])
    }
}

pub struct BrokenObject {
    error: String,
}

#[Object]
impl BrokenObject {
    async fn broken_field(&self) -> FieldResult<String> {
        Err(self.error.clone().into())
    }
}
