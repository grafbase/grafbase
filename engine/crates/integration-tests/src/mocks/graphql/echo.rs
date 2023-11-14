use async_graphql::{EmptyMutation, EmptySubscription, InputObject, Json, Object, ID};

/// A schema that just echoes stuff back at you.
///
/// Useful for testing inputs & outputs
pub type EchoSchema = async_graphql::Schema<Query, EmptyMutation, EmptySubscription>;

#[derive(Default)]
pub struct Query;

#[Object]
impl Query {
    async fn string(&self, input: String) -> String {
        input
    }

    async fn int(&self, input: u32) -> u32 {
        input
    }

    async fn float(&self, input: f32) -> f32 {
        input
    }

    async fn id(&self, input: ID) -> ID {
        input
    }

    async fn list_of_strings(&self, input: Vec<String>) -> Vec<String> {
        input
    }

    async fn list_of_list_of_strings(&self, input: Vec<Vec<String>>) -> Vec<Vec<String>> {
        input
    }

    async fn optional_list_of_optional_strings(
        &self,
        input: Option<Vec<Option<Vec<String>>>>,
    ) -> Option<Vec<Option<Vec<String>>>> {
        input
    }

    async fn input_object(&self, input: InputObj) -> Json<InputObj> {
        Json(input)
    }
}

#[derive(InputObject, serde::Serialize)]
struct InputObj {
    string: Option<String>,
    int: Option<u32>,
    float: Option<f32>,
    id: Option<ID>,
    annoyingly_optional_strings: Option<Vec<Option<Vec<String>>>>,
    recursive_object: Option<Box<InputObj>>,
    recursive_object_list: Vec<InputObj>,
}
