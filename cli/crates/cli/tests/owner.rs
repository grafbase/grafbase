mod utils;

use crate::utils::consts::{
    OWNER_TODO_CREATE, OWNER_TODO_DELETE, OWNER_TODO_GET, OWNER_TODO_LIST, OWNER_TODO_SCHEMA, OWNER_TODO_UPDATE,
};
use json_dotpath::DotPaths;
use serde_json::{json, Value};
use utils::environment::Environment;

/*
All JWTs were generated using header:
```json
{
  "alg": "HS256",
  "typ": "JWT"
}
```
and signature `abc123`
*/

/*
{
  "iss": "https://idp.example.com",
  "exp": 3000000000,
  "iat": 1516239022,
  "sub": "user1"
}
*/
const USER1: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJodHRwczovL2lkcC5leGFtcGxlLmNvbSIsImV4cCI6MzAwMDAwMDAwMCwiaWF0IjoxNTE2MjM5MDIyLCJzdWIiOiJ1c2VyMSJ9.GmXb5LgkrN72MqxdTUKUIWgYlMRTO4WJdQebAghCyXk";
/*
{
  "iss": "https://idp.example.com",
  "exp": 3000000000,
  "iat": 1516239022,
  "sub": "user2"
}
*/
const USER2: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJodHRwczovL2lkcC5leGFtcGxlLmNvbSIsImV4cCI6MzAwMDAwMDAwMCwiaWF0IjoxNTE2MjM5MDIyLCJzdWIiOiJ1c2VyMiJ9.J8j7tjrjd-WaRFcxRBUjev0-1uifRnE0IVt_W-IXdHM";

mod global {
    use super::*;

    mod todo {
        use super::*;
        #[test]
        fn entity_should_be_visible_only_to_the_owner() {
            let mut env = Environment::init(4027);
            env.grafbase_init();
            env.write_schema(OWNER_TODO_SCHEMA);
            env.grafbase_dev();
            let client = env.create_client();
            client.poll_endpoint(30, 300);

            // user1 creates a todo.
            let todo_created = client
                .gql::<Value>(OWNER_TODO_CREATE)
                .bearer(USER1)
                .variables(json!({ "title": "1", "complete": false }))
                .send();

            insta::assert_json_snapshot!("user1-create", todo_created, {".data.todoCreate.todo.id" => "[id]"});
            let id: String = todo_created
                .dot_get("data.todoCreate.todo.id")
                .unwrap()
                .expect("id must be present");
            // user1.list should show the todo.
            insta::assert_json_snapshot!("user1-list", client.gql::<Value>(OWNER_TODO_LIST).bearer(USER1).send());
            // user1 should be able to get the todo by id.
            insta::assert_json_snapshot!(
                "user1-get",
                client
                    .gql::<Value>(OWNER_TODO_GET)
                    .bearer(USER1)
                    .variables(json!({ "id": id }))
                    .send()
            );
            // user1 updates the todo.
            insta::assert_json_snapshot!(
                "user1-update",
                client
                    .gql::<Value>(OWNER_TODO_UPDATE)
                    .bearer(USER1)
                    .variables(json!({"id": id, "complete": true}))
                    .send()
            );
            // user1.list should show the todo with updated complete status.
            insta::assert_json_snapshot!(
                "user1-list-2",
                client.gql::<Value>(OWNER_TODO_LIST).bearer(USER1).send()
            );
            // user2.list should be empty.
            insta::assert_json_snapshot!("list-empty", client.gql::<Value>(OWNER_TODO_LIST).bearer(USER2).send());
            // user2 should not be able to get the todo by id.
            insta::assert_json_snapshot!(
                "user2-get-fail",
                client
                    .gql::<Value>(OWNER_TODO_GET)
                    .bearer(USER2)
                    .variables(json!({ "id": id }))
                    .send()
            );
            // an attempt by user2 to update the todo should fail.
            client
                .gql::<Value>(OWNER_TODO_UPDATE)
                .bearer(USER2)
                .variables(json!({"id": id, "complete": false}))
                .send();
            insta::assert_json_snapshot!(
                "user1-list-2",
                client.gql::<Value>(OWNER_TODO_LIST).bearer(USER1).send()
            );
            // an attemt by user2 to delete the todo should fail.
            client
                .gql::<Value>(OWNER_TODO_DELETE)
                .bearer(USER2)
                .variables(json!({ "id": id }))
                .send();
            insta::assert_json_snapshot!(
                "user1-list-2",
                client.gql::<Value>(OWNER_TODO_LIST).bearer(USER1).send()
            );
            // user1 deletes the todo.
            insta::assert_json_snapshot!(
                "user1-delete",
                client
                    .gql::<Value>(OWNER_TODO_DELETE)
                    .bearer(USER1)
                    .variables(json!({ "id": id }))
                    .send()
            );
            // list of todos should be empty.
            insta::assert_json_snapshot!("list-empty", client.gql::<Value>(OWNER_TODO_LIST).bearer(USER1).send());
        }
    }
}
