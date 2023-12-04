use async_graphql::{EmptyMutation, EmptySubscription, InputObject, Json, MaybeUndefined, Object, ID};

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
        input: Option<Vec<Option<String>>>,
    ) -> Option<Vec<Option<String>>> {
        input
    }

    async fn input_object(&self, input: InputObj) -> Json<InputObj> {
        Json(input)
    }

    async fn list_of_input_object(&self, input: InputObj) -> Json<InputObj> {
        Json(input)
    }
}

#[derive(InputObject, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct InputObj {
    #[serde(skip_serializing_if = "MaybeUndefined::is_undefined")]
    string: MaybeUndefined<String>,
    #[serde(skip_serializing_if = "MaybeUndefined::is_undefined")]
    int: MaybeUndefined<u32>,
    #[serde(skip_serializing_if = "MaybeUndefined::is_undefined")]
    float: MaybeUndefined<f32>,
    #[serde(skip_serializing_if = "MaybeUndefined::is_undefined")]
    id: MaybeUndefined<ID>,
    #[serde(skip_serializing_if = "MaybeUndefined::is_undefined")]
    annoyingly_optional_strings: MaybeUndefined<Vec<Option<Vec<Option<String>>>>>,
    #[serde(skip_serializing_if = "MaybeUndefined::is_undefined")]
    recursive_object: MaybeUndefined<Box<InputObj>>,
    #[serde(skip_serializing_if = "MaybeUndefined::is_undefined")]
    recursive_object_list: MaybeUndefined<Vec<InputObj>>,
}
