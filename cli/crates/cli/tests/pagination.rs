#![allow(unused_crate_dependencies)]
#![allow(clippy::too_many_lines, clippy::panic)]

mod utils;

use backend::project::GraphType;
use serde_json::{json, Value};
use utils::client::Client;
use utils::consts::{
    PAGINATION_CREATE_TODO, PAGINATION_CREATE_TODO_LIST, PAGINATION_PAGINATE_TODOS, PAGINATION_PAGINATE_TODO_LISTS,
    PAGINATION_SCHEMA,
};
use utils::environment::Environment;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Collection<N> {
    page_info: PageInfo,
    edges: Vec<Edge<N>>,
}

#[derive(Debug, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct PageInfo {
    has_next_page: bool,
    has_previous_page: bool,
    start_cursor: Option<String>,
    end_cursor: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct Edge<N> {
    cursor: String,
    node: N,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize)]
struct Todo {
    id: String,
    title: String,
    complete: bool,
}

fn generate_todos(client: &Client, n: usize) -> Vec<Todo> {
    (0..n).fold(Vec::new(), |mut buffer, number| {
        let response = client
            .gql::<Value>(PAGINATION_CREATE_TODO)
            .variables(json!({ "title": format!("Todo#{number}") }))
            .send();

        buffer.push(dot_get!(response, "data.todoCreate.todo", Todo));
        buffer
    })
}

#[test]
fn pagination() {
    let mut env = Environment::init();
    env.grafbase_init(GraphType::Single);
    env.write_schema(PAGINATION_SCHEMA);
    env.grafbase_dev();
    let client = env.create_client().with_api_key();
    client.poll_endpoint(30, 300);

    let todos = generate_todos(&client, 3);

    for number in 0..3 {
        client
            .gql::<Value>(PAGINATION_CREATE_TODO_LIST)
            .variables(json!({
                "title": (number + 1).to_string() ,
                "todo0": todos[0].id,
                "todo1": todos[1].id,
                "todo2": todos[2].id,
            }))
            .send();
    }

    for (query, path) in &[
        (PAGINATION_PAGINATE_TODOS, "data.todoCollection"),
        (
            PAGINATION_PAGINATE_TODO_LISTS,
            "data.todoListCollection.edges.0.node.todos",
        ),
    ] {
        let todo_collection = |variables: Value| {
            let response = client.gql::<Value>(*query).variables(variables).send();
            dot_get!(response, path, Collection<Todo>)
        };

        //
        // last
        //
        let Collection { page_info, edges } = todo_collection(json!({"last": 1}));
        let [Edge {
            cursor: last_cursor,
            node: last_todo,
        }] = &edges[..]
        else {
            panic!("Expected exactly one edge {edges:?}");
        };
        assert_eq!(last_todo, &todos[2]);
        assert_eq!(
            page_info,
            PageInfo {
                has_next_page: false,
                has_previous_page: true,
                start_cursor: Some(last_cursor.clone()),
                end_cursor: Some(last_cursor.clone())
            }
        );

        //
        // last, before
        //
        let Collection { page_info, edges } = todo_collection(json!({"last": 10, "before": last_cursor}));
        let [Edge {
            cursor: first_cursor,
            node: first_todo,
        }, Edge {
            cursor: middle_cursor,
            node: middle_todo,
        }] = &edges[..]
        else {
            panic!("Expected exactly one edge {edges:?}");
        };

        assert_eq!(first_todo, &todos[0]);
        assert_eq!(middle_todo, &todos[1]);
        assert_eq!(
            page_info,
            PageInfo {
                has_next_page: false,
                has_previous_page: false,
                start_cursor: Some(first_cursor.clone()),
                end_cursor: Some(middle_cursor.clone())
            }
        );

        //
        // first
        //
        let Collection { page_info, edges } = todo_collection(json!({"first": 1}));
        let [Edge {
            cursor: first_cursor,
            node: first_todo,
        }] = &edges[..]
        else {
            panic!("Expected exactly one edge {edges:?}");
        };
        assert_eq!(first_todo, &todos[0]);
        assert_eq!(
            page_info,
            PageInfo {
                has_next_page: true,
                has_previous_page: false,
                start_cursor: Some(first_cursor.clone()),
                end_cursor: Some(first_cursor.clone())
            }
        );

        //
        // first, after
        //
        let Collection { page_info, edges } = todo_collection(json!({"first": 1, "after": first_cursor}));
        let [Edge {
            cursor: middle_cursor,
            node: middle_todo,
        }] = &edges[..]
        else {
            panic!("Expected exactly one edge {edges:?}");
        };
        assert_eq!(middle_todo, &todos[1]);
        assert_eq!(
            page_info,
            PageInfo {
                has_next_page: true,
                // The Relay spec: https://relay.dev/graphql/connections.htm#sec-Connection-Types.Fields.PageInfo
                // defines that has_previous_page is set "If the server can efficiently determine that elements
                // exist prior to after, return true." Currently we don't, so we don't test the
                // value.
                has_previous_page: page_info.has_previous_page,
                start_cursor: Some(middle_cursor.clone()),
                end_cursor: Some(middle_cursor.clone())
            }
        );

        let Collection { page_info, edges } = todo_collection(json!({"first": 1, "after": middle_cursor}));
        let [Edge {
            cursor: last_cursor,
            node: last_todo,
        }] = &edges[..]
        else {
            panic!("Expected exactly one edge {edges:?}");
        };
        assert_eq!(last_todo, &todos[2]);
        assert_eq!(
            page_info,
            PageInfo {
                has_next_page: false,
                // See previous comment for the previous test.
                has_previous_page: page_info.has_previous_page,
                start_cursor: Some(last_cursor.clone()),
                end_cursor: Some(last_cursor.clone())
            }
        );
    }
}

macro_rules! assert_same_todos {
    ($collection: expr, $expected: expr) => {{
        let result = $collection
            .edges
            .iter()
            .map(|edge| edge.node.clone())
            .collect::<Vec<_>>();
        assert_eq!(result, Vec::from($expected));
    }};
}

#[test]
fn pagination_order() {
    let mut env = Environment::init();
    env.grafbase_init(GraphType::Single);
    env.write_schema(PAGINATION_SCHEMA);
    env.grafbase_dev();
    let client = env.create_client().with_api_key();
    client.poll_endpoint(30, 300);

    let todos = generate_todos(&client, 5);
    let reversed_todos = {
        let mut tmp = todos.clone();
        tmp.reverse();
        tmp
    };

    let todo_collection = |variables: Value| {
        let response = client
            .gql::<Value>(PAGINATION_PAGINATE_TODOS)
            .variables(variables)
            .send();
        dot_get!(response, "data.todoCollection", Collection<Todo>)
    };

    /////////
    // ASC //
    /////////
    let all_asc = todo_collection(json!({
        "first": 100,
        "orderBy": { "createdAt": "ASC" }
    }));
    assert_same_todos!(all_asc, &todos[..]);
    assert_same_todos!(
        todo_collection(json!({
            "last": 100,
            "orderBy": { "createdAt": "ASC" }
        })),
        &todos[..]
    );

    // FIRST
    assert_same_todos!(
        todo_collection(json!({
            "first": 3,
            "orderBy": { "createdAt": "ASC" }
        })),
        &todos[..3]
    );
    let first_cursor = all_asc.edges.first().unwrap().cursor.clone();
    assert_same_todos!(
        todo_collection(json!({
            "first": 2,
            "after": first_cursor,
            "orderBy": { "createdAt": "ASC" }
        })),
        &todos[1..3]
    );

    // LAST
    assert_same_todos!(
        todo_collection(json!({
            "last": 3,
            "orderBy": { "createdAt": "ASC" }
        })),
        &todos[2..]
    );
    let last_cursor = all_asc.edges.last().unwrap().cursor.clone();
    assert_same_todos!(
        todo_collection(json!({
            "last": 2,
            "before": last_cursor,
            "orderBy": { "createdAt": "ASC" }
        })),
        &todos[2..4]
    );

    //////////
    // DESC //
    //////////
    let all_desc = todo_collection(json!({
        "first": 100,
        "orderBy": { "createdAt": "DESC" }
    }));
    assert_same_todos!(all_desc, &reversed_todos[..]);
    assert_same_todos!(
        todo_collection(json!({
            "last": 100,
            "orderBy": { "createdAt": "DESC" }
        })),
        &reversed_todos[..]
    );

    // FIRST
    assert_same_todos!(
        todo_collection(json!({
            "first": 3,
            "orderBy": { "createdAt": "DESC" }
        })),
        &reversed_todos[..3]
    );
    let first_cursor = all_desc.edges.first().unwrap().cursor.clone();
    assert_same_todos!(
        todo_collection(json!({
            "first": 2,
            "after": first_cursor,
            "orderBy": { "createdAt": "DESC" }
        })),
        &reversed_todos[1..3]
    );

    // LAST
    assert_same_todos!(
        todo_collection(json!({
            "last": 3,
            "orderBy": { "createdAt": "DESC" }

        })),
        &reversed_todos[2..]
    );
    let last_cursor = all_desc.edges.last().unwrap().cursor.clone();
    assert_same_todos!(
        todo_collection(json!({
            "last": 2,
            "before": last_cursor,
            "orderBy": { "createdAt": "DESC" }
        })),
        &reversed_todos[2..4]
    );
}

#[test]
fn collection_id_filter() {
    let mut env = Environment::init();
    env.grafbase_init(GraphType::Single);
    env.write_schema(PAGINATION_SCHEMA);
    env.grafbase_dev();
    let client = env.create_client().with_api_key();
    client.poll_endpoint(30, 300);

    let mut todos = generate_todos(&client, 5);
    let todo_collection = |variables: Value| {
        let response = client
            .gql::<Value>(PAGINATION_PAGINATE_TODOS)
            .variables(variables)
            .send();
        dot_get!(response, "data.todoCollection", Collection<Todo>)
    };
    let mut todo_ids = todos.iter().map(|todo| todo.id.clone()).collect::<Vec<_>>();

    // all ids
    assert_same_todos!(
        todo_collection(json!({
            "first": 100,
            "filter": { "id": { "in": todo_ids } }
        })),
        &todos[..]
    );
    // single one
    assert_same_todos!(
        todo_collection(json!({
            "first": 100,
            "filter": { "id": { "in": [todo_ids[0]] } }
        })),
        &todos[..1]
    );
    // duplicate
    assert_same_todos!(
        todo_collection(json!({
            "first": 100,
            "filter": { "id": { "in": [todo_ids[0], todo_ids[0]] } }
        })),
        &todos[..1]
    );

    // non-existant
    assert_same_todos!(
        todo_collection(json!({
            "first": 100,
            "filter": { "id": { "in": [todo_ids[0], "random"] } }
        })),
        &todos[..1]
    );

    //
    // ORDER BY ASC
    //
    assert_same_todos!(
        todo_collection(json!({
            "first": 100,
            "filter": { "id": { "in": todo_ids } },
            "orderBy": { "createdAt": "ASC" }
        })),
        &todos[..]
    );
    assert_same_todos!(
        todo_collection(json!({
            "last": 100,
            "filter": { "id": { "in": todo_ids } },
            "orderBy": { "createdAt": "ASC" }
        })),
        &todos[..]
    );

    let page = todo_collection(json!({
        "first": 3,
        "filter": { "id": { "in": todo_ids } },
        "orderBy": { "createdAt": "ASC" }
    }));
    assert_same_todos!(page, &todos[..3]);
    assert_same_todos!(
        todo_collection(json!({
            "first": 3,
            "after": page.edges.last().unwrap().cursor,
            "filter": { "id": { "in": todo_ids } },
            "orderBy": { "createdAt": "ASC" }
        })),
        &todos[3..]
    );

    let page = todo_collection(json!({
        "last": 3,
        "filter": { "id": { "in": todo_ids } },
        "orderBy": { "createdAt": "ASC" }
    }));
    assert_same_todos!(page, &todos[2..]);
    assert_same_todos!(
        todo_collection(json!({
            "last": 3,
            "before": page.edges.first().unwrap().cursor,
            "filter": { "id": { "in": todo_ids } },
            "orderBy": { "createdAt": "ASC" }
        })),
        &todos[..2]
    );

    //
    // ORDER BY DESC
    //
    todos.reverse();
    todo_ids.reverse();
    assert_same_todos!(
        todo_collection(json!({
            "first": 100,
            "filter": { "id": { "in": todo_ids } },
            "orderBy": { "createdAt": "DESC" }
        })),
        &todos[..]
    );
    assert_same_todos!(
        todo_collection(json!({
            "last": 100,
            "filter": { "id": { "in": todo_ids } },
            "orderBy": { "createdAt": "DESC" }
        })),
        &todos[..]
    );

    let page = todo_collection(json!({
        "first": 3,
        "filter": { "id": { "in": todo_ids } },
        "orderBy": { "createdAt": "DESC" }
    }));
    assert_same_todos!(page, &todos[..3]);
    assert_same_todos!(
        todo_collection(json!({
            "first": 3,
            "after": page.edges.last().unwrap().cursor,
            "filter": { "id": { "in": todo_ids } },
            "orderBy": { "createdAt": "DESC" }
        })),
        &todos[3..]
    );

    let page = todo_collection(json!({
        "last": 3,
        "filter": { "id": { "in": todo_ids } },
        "orderBy": { "createdAt": "DESC" }
    }));
    assert_same_todos!(page, &todos[2..]);
    assert_same_todos!(
        todo_collection(json!({
            "last": 3,
            "before": page.edges.first().unwrap().cursor,
            "filter": { "id": { "in": todo_ids } },
            "orderBy": { "createdAt": "DESC" }
        })),
        &todos[..2]
    );
}
