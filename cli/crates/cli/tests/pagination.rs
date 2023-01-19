#![allow(clippy::too_many_lines)]

mod utils;

use serde_json::{json, Value};
use utils::consts::{
    PAGINATION_CREATE_TODO, PAGINATION_CREATE_TODO_LIST, PAGINATION_PAGINATE_TODOS, PAGINATION_PAGINATE_TODO_LISTS,
    PAGINATION_SCHEMA,
};
use utils::environment::Environment;

#[test]
fn pagination() {
    let mut env = Environment::init(4010);
    env.grafbase_init();
    env.write_schema(PAGINATION_SCHEMA);
    env.grafbase_dev();

    let client = env.create_client();
    client.poll_endpoint(30, 300);
    let todo_ids = (0..3).fold(Vec::new(), |mut buffer, number| {
        let response = client.gql::<Value>(
            json!({
                "query": PAGINATION_CREATE_TODO,
                "variables": { "title": number.to_string() }
            })
            .to_string(),
        );
        buffer.push(dot_get!(dbg!(response), "data.todoCreate.todo.id", String));
        buffer
    });

    for number in 0..3 {
        client.gql::<Value>(
            json!({
                "query": PAGINATION_CREATE_TODO_LIST,
                "variables": {
                "title": (number + 1).to_string() ,
                "todo0": todo_ids[0],
                "todo1": todo_ids[1],
                "todo2": todo_ids[2],
            }
            })
            .to_string(),
        );
    }

    for (query, prefix) in &[
        (PAGINATION_PAGINATE_TODOS, "data.todoCollection"),
        (
            PAGINATION_PAGINATE_TODO_LISTS,
            "data.todoListCollection.edges.0.node.todos",
        ),
    ] {
        //
        // last
        //
        let response = client.gql::<Value>(
            json!({
                "query": query,
                "variables": {
                    "last": 1
                }
            })
            .to_string(),
        );

        let last_todo: Value = dot_get!(response, &format!("{prefix}.edges.0.node"));
        let last_todo_id: String = dot_get!(last_todo, "id");
        let last_todo_title: String = dot_get!(last_todo, "title");
        assert_eq!(last_todo_id, todo_ids[2]);
        assert_eq!(last_todo_title, "2");

        let has_next_page: bool = dot_get!(response, &format!("{prefix}.pageInfo.hasNextPage"));
        let has_previous_page: bool = dot_get!(response, &format!("{prefix}.pageInfo.hasPreviousPage"));
        assert!(!has_next_page);
        assert!(has_previous_page);

        let last_cursor: String = dot_get!(response, &format!("{prefix}.edges.0.cursor"));

        //
        // last, before
        //
        let response = client.gql::<Value>(
            json!({
                "query": query,
                "variables": {
                    "last": 10,
                    "before": last_cursor
                }
            })
            .to_string(),
        );

        let previous_todo: Value = dot_get!(response, &format!("{prefix}.edges.1.node"));
        let previous_todo_id: String = dot_get!(previous_todo, "id");
        let previous_todo_title: String = dot_get!(previous_todo, "title");
        assert!(previous_todo_id != last_todo_id);
        assert_eq!(previous_todo_title, "1");

        let has_next_page: bool = dot_get!(response, &format!("{prefix}.pageInfo.hasNextPage"));
        let has_previous_page: bool = dot_get!(response, &format!("{prefix}.pageInfo.hasPreviousPage"));
        assert!(!has_next_page);
        assert!(!has_previous_page);

        //
        // first
        //
        let response = client.gql::<Value>(
            json!({
                "query": query,
                "variables": {
                    "first": 1
                }
            })
            .to_string(),
        );

        let first_todo: Value = dot_get!(response, &format!("{prefix}.edges.0.node"));
        let first_todo_id: String = dot_get!(first_todo, "id");
        let first_todo_title: String = dot_get!(first_todo, "title");
        assert_eq!(first_todo_id, todo_ids[0]);
        assert_eq!(first_todo_title, "0");

        let first_cursor: String = dot_get!(response, &format!("{prefix}.edges.0.cursor"));

        //
        // first, after
        //
        let response = client.gql::<Value>(
            json!({
                "query": query,
                "variables": {
                    "first": 1,
                    "after": first_cursor
                }
            })
            .to_string(),
        );

        let next_todo: Value = dot_get!(response, &format!("{prefix}.edges.0.node"));
        let has_next_page: bool = dot_get!(response, &format!("{prefix}.pageInfo.hasNextPage"));
        let has_previous_page: bool = dot_get!(response, &format!("{prefix}.pageInfo.hasPreviousPage"));
        assert!(has_next_page);
        assert!(!has_previous_page);

        let next_todo_id: String = dot_get!(next_todo, "id");
        let next_todo_title: String = dot_get!(next_todo, "title");
        assert!(next_todo_id != first_todo_id);
        assert_eq!(next_todo_title, "1");

        let response = client.gql::<Value>(
            json!({
                "query": query,
                "variables": {
                    "first": 10,
                    "after": first_cursor
                }
            })
            .to_string(),
        );

        let next_todo: Value = dot_get!(response, &format!("{prefix}.edges.0.node"));
        let next_todo_id: String = dot_get!(next_todo, "id");
        let next_todo_title: String = dot_get!(next_todo, "title");
        assert!(next_todo_id != first_todo_id);
        assert_eq!(next_todo_title, "1");

        let has_next_page: bool = dot_get!(response, &format!("{prefix}.pageInfo.hasNextPage"));
        let has_previous_page: bool = dot_get!(response, &format!("{prefix}.pageInfo.hasPreviousPage"));
        assert!(!has_next_page);
        assert!(!has_previous_page);
    }
}
