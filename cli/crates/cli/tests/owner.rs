#![allow(unused_crate_dependencies)]

use chrono::Duration;
use serde_json::json;

mod utils;

mod global {

    mod todo {
        use crate::utils::consts::{
            OWNER_TODO_CREATE, OWNER_TODO_DELETE, OWNER_TODO_GET, OWNER_TODO_LIST, OWNER_TODO_MIXED_SCHEMA,
            OWNER_TODO_OWNER_CREATE_SCHEMA, OWNER_TODO_SCHEMA, OWNER_TODO_UPDATE,
        };
        use crate::utils::environment::Environment;
        use crate::{admin_jwt, user_one_jwt, user_three_jwt, user_two_jwt};
        use backend::project::GraphType;
        use json_dotpath::DotPaths;
        use serde_json::{json, Value};

        #[ignore]
        #[test]
        fn entity_should_be_visible_only_to_the_owner() {
            let mut env = Environment::init();
            env.grafbase_init(GraphType::Single);
            env.write_schema(OWNER_TODO_SCHEMA);
            env.grafbase_dev();
            let client = env.create_client();
            client.poll_endpoint(30, 300);

            // user1 creates a todo.
            let todo_created = client
                .gql::<Value>(OWNER_TODO_CREATE)
                .bearer(&user_one_jwt())
                .variables(json!({ "title": "1", "complete": false }))
                .send();

            insta::assert_json_snapshot!("user1-create", todo_created, {".data.todoCreate.todo.id" => "[id]"});
            let id: String = todo_created
                .dot_get("data.todoCreate.todo.id")
                .unwrap()
                .expect("id must be present");
            // user1.list should see the todo.
            insta::assert_json_snapshot!(
                "user1-list",
                client.gql::<Value>(OWNER_TODO_LIST).bearer(&user_one_jwt()).send()
            );
            // user1 should be able to get the todo by id.
            insta::assert_json_snapshot!(
                "user1-get",
                client
                    .gql::<Value>(OWNER_TODO_GET)
                    .bearer(&user_one_jwt())
                    .variables(json!({ "id": id }))
                    .send()
            );
            // user1 updates the todo.
            insta::assert_json_snapshot!(
                "user1-update",
                client
                    .gql::<Value>(OWNER_TODO_UPDATE)
                    .bearer(&user_one_jwt())
                    .variables(json!({"id": id, "input": { "complete": true }}))
                    .send()
            );
            // user1.list should see the todo with updated complete status.
            insta::assert_json_snapshot!(
                "user1-list-2",
                client.gql::<Value>(OWNER_TODO_LIST).bearer(&user_one_jwt()).send()
            );
            // user2.list should be empty.
            insta::assert_json_snapshot!(
                "list-empty",
                client.gql::<Value>(OWNER_TODO_LIST).bearer(&user_two_jwt()).send()
            );
            // user2 should not be able to get the todo by id.
            insta::assert_json_snapshot!(
                "user2-get-fail",
                client
                    .gql::<Value>(OWNER_TODO_GET)
                    .bearer(&user_two_jwt())
                    .variables(json!({ "id": id }))
                    .send()
            );
            // an attempt by user2 to update the todo should fail.
            client
                .gql::<Value>(OWNER_TODO_UPDATE)
                .bearer(&user_two_jwt())
                .variables(json!({"id": id, "input": { "complete": false }}))
                .send();
            insta::assert_json_snapshot!(
                "user1-list-2",
                client.gql::<Value>(OWNER_TODO_LIST).bearer(&user_one_jwt()).send()
            );
            // an attemt by user2 to delete the todo should fail.
            client
                .gql::<Value>(OWNER_TODO_DELETE)
                .bearer(&user_two_jwt())
                .variables(json!({ "id": id }))
                .send();
            insta::assert_json_snapshot!(
                "user1-list-2",
                client.gql::<Value>(OWNER_TODO_LIST).bearer(&user_one_jwt()).send()
            );
            // user1 deletes the todo.
            insta::assert_json_snapshot!(
                "user1-delete",
                client
                    .gql::<Value>(OWNER_TODO_DELETE)
                    .bearer(&user_one_jwt())
                    .variables(json!({ "id": id }))
                    .send()
            );
            // list of todos should be empty.
            insta::assert_json_snapshot!(
                "list-empty",
                client.gql::<Value>(OWNER_TODO_LIST).bearer(&user_one_jwt()).send()
            );
        }

        #[ignore]
        #[test]
        fn owner_create_group_all_should_work() {
            let mut env = Environment::init();
            env.grafbase_init(GraphType::Single);
            env.write_schema(OWNER_TODO_OWNER_CREATE_SCHEMA);
            env.grafbase_dev();
            let client = env.create_client();
            client.poll_endpoint(30, 300);

            // user1 creates a todo.
            let todo_created = client
                .gql::<Value>(OWNER_TODO_CREATE)
                .bearer(&user_one_jwt())
                .variables(json!({ "title": "1", "complete": false }))
                .send();
            insta::assert_json_snapshot!("user1-create", todo_created, {".data.todoCreate.todo.id" => "[id]"});
            let id: String = todo_created
                .dot_get("data.todoCreate.todo.id")
                .unwrap()
                .expect("id must be present");

            // // admin.list should see the todo.
            insta::assert_json_snapshot!("list", client.gql::<Value>(OWNER_TODO_LIST).bearer(&admin_jwt()).send());
            // user3.list should see the todo.
            insta::assert_json_snapshot!(
                "list",
                client.gql::<Value>(OWNER_TODO_LIST).bearer(&user_three_jwt()).send()
            );
            // user1.list should be unauthorized.
            insta::assert_json_snapshot!(
                "list-fail",
                client.gql::<Value>(OWNER_TODO_LIST).bearer(&user_one_jwt()).send()
            );
            // user2.list should be unauthorized.
            insta::assert_json_snapshot!(
                "list-fail",
                client.gql::<Value>(OWNER_TODO_LIST).bearer(&user_two_jwt()).send()
            );

            // user1 should be able to get the todo by id.
            insta::assert_json_snapshot!(
                "get",
                client
                    .gql::<Value>(OWNER_TODO_GET)
                    .bearer(&user_one_jwt())
                    .variables(json!({ "id": id }))
                    .send()
            );
            // user2 should not be able to get the todo by id.
            insta::assert_json_snapshot!(
                "get-fail",
                client
                    .gql::<Value>(OWNER_TODO_GET)
                    .bearer(&user_two_jwt())
                    .variables(json!({ "id": id }))
                    .send()
            );
            // admin should be able to get the todo by id.
            insta::assert_json_snapshot!(
                "get",
                client
                    .gql::<Value>(OWNER_TODO_GET)
                    .bearer(&admin_jwt())
                    .variables(json!({ "id": id }))
                    .send()
            );
            // user3 should be able to get the todo by id.
            insta::assert_json_snapshot!(
                "get",
                client
                    .gql::<Value>(OWNER_TODO_GET)
                    .bearer(&user_three_jwt())
                    .variables(json!({ "id": id }))
                    .send()
            );
        }

        #[ignore]
        #[test]
        fn group_should_supercede_owner_when_listing_entities() {
            let mut env = Environment::init();
            env.grafbase_init(GraphType::Single);
            env.write_schema(OWNER_TODO_MIXED_SCHEMA);
            env.grafbase_dev();
            let client = env.create_client();
            client.poll_endpoint(30, 300);

            // user1 creates a todo.
            let todo_created = client
                .gql::<Value>(OWNER_TODO_CREATE)
                .bearer(&user_one_jwt())
                .variables(json!({ "title": "1", "complete": false }))
                .send();
            insta::assert_json_snapshot!("user1-create", todo_created, {".data.todoCreate.todo.id" => "[id]"});
            let id: String = todo_created
                .dot_get("data.todoCreate.todo.id")
                .unwrap()
                .expect("id must be present");

            // admin.list should see the todo.
            insta::assert_json_snapshot!("list", client.gql::<Value>(OWNER_TODO_LIST).bearer(&admin_jwt()).send());
            // user3.list should see the todo.
            insta::assert_json_snapshot!(
                "list",
                client.gql::<Value>(OWNER_TODO_LIST).bearer(&user_three_jwt()).send()
            );
            // user1.list should see the todo.
            insta::assert_json_snapshot!(
                "list",
                client.gql::<Value>(OWNER_TODO_LIST).bearer(&user_one_jwt()).send()
            );
            // user2.list should be unauthorized.
            insta::assert_json_snapshot!(
                "list",
                client.gql::<Value>(OWNER_TODO_LIST).bearer(&user_one_jwt()).send()
            );

            // user1 should see the todo.
            insta::assert_json_snapshot!(
                "get",
                client
                    .gql::<Value>(OWNER_TODO_GET)
                    .bearer(&user_one_jwt())
                    .variables(json!({ "id": id }))
                    .send()
            );
            // user2 should not be able to get the todo by id.
            insta::assert_json_snapshot!(
                "get-fail",
                client
                    .gql::<Value>(OWNER_TODO_GET)
                    .bearer(&user_two_jwt())
                    .variables(json!({ "id": id }))
                    .send()
            );
            // admin should see the todo.
            insta::assert_json_snapshot!(
                "get",
                client
                    .gql::<Value>(OWNER_TODO_GET)
                    .bearer(&admin_jwt())
                    .variables(json!({ "id": id }))
                    .send()
            );
            // user3 should see the todo.
            insta::assert_json_snapshot!(
                "get",
                client
                    .gql::<Value>(OWNER_TODO_GET)
                    .bearer(&user_three_jwt())
                    .variables(json!({ "id": id }))
                    .send()
            );
        }
    }

    mod twitter {
        use crate::utils::consts::{
            OWNER_TWITTER_SCHEMA, OWNER_TWITTER_TWEET_CREATE, OWNER_TWITTER_USER_AND_TWEETS_GET_BY_ID,
            OWNER_TWITTER_USER_CREATE, OWNER_TWITTER_USER_GET_BY_EMAIL, OWNER_TWITTER_USER_GET_BY_ID,
        };
        use crate::utils::environment::Environment;
        use crate::{user_one_jwt, user_two_jwt};
        use backend::project::GraphType;
        use json_dotpath::DotPaths;
        use serde_json::{json, Value};

        #[ignore]
        #[test]
        fn get_by_id_should_be_filtered_by_the_owner() {
            let mut env = Environment::init();
            env.grafbase_init(GraphType::Single);
            env.write_schema(OWNER_TWITTER_SCHEMA);
            env.grafbase_dev();
            let client = env.create_client();
            client.poll_endpoint(30, 300);

            // user1 creates a user entity.
            let email: &str = "user1@example.com";
            let user_created = client
                .gql::<Value>(OWNER_TWITTER_USER_CREATE)
                .bearer(&user_one_jwt())
                .variables(
                    json!({ "username": "user1", "email": email, "avatar": "http://example.com", "url": "http://example.com" }),
                )
                .send();

            insta::assert_json_snapshot!("user1-create", user_created, {".data.userCreate.user.id" => "[id]"});
            let id: String = user_created
                .dot_get("data.userCreate.user.id")
                .unwrap()
                .expect("id must be present");
            // user1 can use get by id
            insta::assert_json_snapshot!(
                "user1-get",
                client
                    .gql::<Value>(OWNER_TWITTER_USER_GET_BY_ID)
                    .bearer(&user_one_jwt())
                    .variables(json!({ "id": id }))
                    .send()
            );
            // user2 cannot get the user entity by id
            insta::assert_json_snapshot!(
                "user2-get-empty",
                client
                    .gql::<Value>(OWNER_TWITTER_USER_GET_BY_ID)
                    .bearer(&user_two_jwt())
                    .variables(json!({ "id": id }))
                    .send()
            );
        }

        #[ignore]
        #[test]
        fn get_by_email_should_be_filtered_by_the_owner() {
            let mut env = Environment::init();
            env.grafbase_init(GraphType::Single);
            env.write_schema(OWNER_TWITTER_SCHEMA);
            env.grafbase_dev();
            let client = env.create_client();
            client.poll_endpoint(30, 300);

            // user1 creates a user entity.
            let email: &str = "user1@example.com";
            let user_created = client
                .gql::<Value>(OWNER_TWITTER_USER_CREATE)
                .bearer(&user_one_jwt())
                .variables(
                    json!({ "username": "user1", "email": email, "avatar": "http://example.com", "url": "http://example.com" }),
                )
                .send();

            insta::assert_json_snapshot!("user1-create", user_created, {".data.userCreate.user.id" => "[id]"});
            // user1 can create a tweet
            insta::assert_json_snapshot!(
                "user1-get",
                client
                    .gql::<Value>(OWNER_TWITTER_USER_GET_BY_EMAIL)
                    .bearer(&user_one_jwt())
                    .variables(json!({ "email": email }))
                    .send()
            );
            // user2 cannot get the user entity by email
            insta::assert_json_snapshot!(
                "user2-get-empty",
                client
                    .gql::<Value>(OWNER_TWITTER_USER_GET_BY_EMAIL)
                    .bearer(&user_two_jwt())
                    .variables(json!({ "email": email }))
                    .send()
            );
        }

        #[ignore]
        #[test]
        fn test_linking() {
            let mut env = Environment::init();
            env.grafbase_init(GraphType::Single);
            env.write_schema(OWNER_TWITTER_SCHEMA);
            env.grafbase_dev();
            let client = env.create_client();
            client.poll_endpoint(30, 300);

            // user1 creates a user entity.
            let email: &str = "user1@example.com";
            let user_created = client
                .gql::<Value>(OWNER_TWITTER_USER_CREATE)
                .bearer(&user_one_jwt())
                .variables(
                    json!({ "username": "user1", "email": email, "avatar": "http://example.com", "url": "http://example.com" }),
                )
                .send();

            insta::assert_json_snapshot!("user1-create", user_created, {".data.userCreate.user.id" => "[id]"});
            let id: String = user_created
                .dot_get("data.userCreate.user.id")
                .unwrap()
                .expect("id must be present");
            // user1 can create a tweet linked to the user entity
            insta::assert_json_snapshot!(
                "user1-create-tweet",
                client
                    .gql::<Value>(OWNER_TWITTER_TWEET_CREATE)
                    .bearer(&user_one_jwt())
                    .variables(json!({ "userId": id }))
                    .send(),
                {".data.tweetCreate.tweet.id" => "[id]"}
            );
            // user2 cannot get the entity by id
            insta::assert_json_snapshot!(
                "user2-create-tweet-fail",
                client
                    .gql::<Value>(OWNER_TWITTER_TWEET_CREATE)
                    .bearer(&user_two_jwt())
                    .variables(json!({ "userId": id }))
                    .send()
            );
            // user1 can use get by id
            insta::assert_json_snapshot!(
                "user1-and-tweets-get",
                client
                    .gql::<Value>(OWNER_TWITTER_USER_AND_TWEETS_GET_BY_ID)
                    .bearer(&user_one_jwt())
                    .variables(json!({ "id": id }))
                    .send()
            );
        }
    }
}

fn user_one_jwt() -> String {
    make_jwt(&json!({
        "iss": "https://idp.example.com",
        "sub": "user1"
    }))
}

fn user_two_jwt() -> String {
    make_jwt(&json!({
        "iss": "https://idp.example.com",
        "sub": "user2"
    }))
}

fn user_three_jwt() -> String {
    make_jwt(&json!({
        "iss": "https://idp.example.com",
        "groups": [
            "admin"
        ],
        "sub": "user3",
    }))
}

fn admin_jwt() -> String {
    make_jwt(&json!({
        "iss": "https://idp.example.com",
        "groups": [
            "admin"
        ],
    }))
}

fn make_jwt(claims: &serde_json::Value) -> String {
    use jwt_compact::{
        alg::{Hs256, Hs256Key},
        AlgorithmExt, Claims, Header, TimeOptions,
    };

    let claims = Claims::new(claims).set_duration_and_issuance(&TimeOptions::default(), Duration::days(7));

    let key = Hs256Key::new(b"abc123");
    let header = Header::empty().with_key_id("my-key");

    Hs256.token(&header, &claims, &key).unwrap()
}
