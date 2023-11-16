#![allow(unused_crate_dependencies)]
#![allow(clippy::too_many_lines)]
mod utils;

use std::collections::{HashMap, HashSet};
use std::thread;
use std::time::Duration;

use rstest::rstest;
use serde_json::{json, Value};
use utils::consts::{
    SEARCH_CREATE_LIST, SEARCH_CREATE_OPTIONAL, SEARCH_CREATE_PERSON, SEARCH_CREATE_REQUIRED, SEARCH_METADATA_FIELDS,
    SEARCH_PAGINATION, SEARCH_SCHEMA, SEARCH_SEARCH_LIST, SEARCH_SEARCH_OPTIONAL, SEARCH_SEARCH_PERSON,
    SEARCH_SEARCH_REQUIRED,
};
use utils::environment::Environment;

macro_rules! assert_same_fields {
    ($result: expr, $expected_id: expr, $expected_vars: expr) => {{
        let result = $result;
        let expected_vars = $expected_vars;
        assert_eq!(dot_get!(result, "id", String), $expected_id);
        for field in &[
            "ip",
            "timestamp",
            "url",
            "email",
            "phone",
            "date",
            "datetime",
            "text",
            "int",
            "float",
            "bool",
        ] {
            let value = dot_get!(result, field, Value);
            let expected = dot_get!(expected_vars, field, Value);
            assert_eq!(value, expected, "Different value for field {}", field);
        }
    }};
}

macro_rules! assert_hits_unordered {
    ($result: expr, $($expected_id:expr),+) => {{
        let expected = HashSet::from([$($expected_id.clone()),+]);
        let result = $result;
        assert_eq!(
            result
                .edges
                .iter()
                .map(|edge| dot_get!(edge.node, "id", String))
                .collect::<HashSet<_>>(),
            expected
        );
    }};
}

macro_rules! assert_hits {
    ($result: expr, $hits: expr, has_next_page: $has_next_page: expr, has_previous_page: $has_previous_page: expr, total_hits: $total_hits: expr) => {
        let result = $result;
        assert_eq!(
            result
                .edges
                .iter()
                .map(|edge| dot_get!(edge.node, "id", String))
                .collect::<Vec<_>>(),
            $hits.iter().map(|hit| hit.clone()).collect::<Vec<_>>(),
        );
        assert_eq!(
            result.page_info.start_cursor,
            result.edges.first().map(|edge| edge.cursor.clone()),
            "start_cursor"
        );
        assert_eq!(
            result.page_info.end_cursor,
            result.edges.last().map(|edge| edge.cursor.clone()),
            "end_cursor"
        );
        assert_eq!(result.page_info.has_next_page, $has_next_page, "has_next_page");
        assert_eq!(
            result.page_info.has_previous_page, $has_previous_page,
            "has_previous_page"
        );
        assert_eq!(result.search_info.total_hits, $total_hits, "total_hits");
    };
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Collection<N> {
    page_info: PageInfo,
    search_info: SearchInfo,
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

#[derive(Debug, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct SearchInfo {
    total_hits: i64,
}

#[derive(Debug, serde::Deserialize)]
struct Edge<N> {
    cursor: String,
    #[allow(dead_code)]
    score: f64,
    node: N,
}

#[cfg(not(feature = "dynamodb"))] // GB-3636
#[test]
fn search_enums() {
    use backend::project::GraphType;

    let mut env = Environment::init();
    env.grafbase_init(GraphType::Single);
    env.write_schema(SEARCH_SCHEMA);
    env.grafbase_dev();
    let client = env.create_client().with_api_key();
    client.poll_endpoint(30, 300);

    let create = |variables: Value| {
        let response = client.gql::<Value>(SEARCH_CREATE_PERSON).variables(variables).send();
        dot_get!(response, "data.personCreate.person.id", String)
    };
    let search = |variables: Value| {
        let response = client.gql::<Value>(SEARCH_SEARCH_PERSON).variables(variables).send();
        dot_get!(response, "data.personSearch", Collection<Value>)
    };
    let filter = |filter: Value| search(json!({"first": 10, "filter": filter}));
    let search_text = |query: &str| search(json!({"first": 10, "query": query}));

    let cat_person = create(json!({"alive": "SCHRODINGER", "favoritePet": "CAT",  "pets": ["CAT", "HAMSTER"]}));
    let dog_person = create(json!({"alive": "YES", "favoritePet": "DOG", "pets": ["DOG", "HAMSTER"]}));
    let dead_person = create(json!({"alive": "NO"}));

    // Filters
    assert_hits_unordered!(filter(json!({"alive": {"eq": "YES"}})), dog_person);
    assert_hits_unordered!(filter(json!({"alive": {"eq": "SCHRODINGER"}})), cat_person);
    assert_hits_unordered!(filter(json!({"alive": {"eq": "NO"}})), dead_person);
    assert_hits_unordered!(
        filter(json!({"alive": {"in": ["YES", "SCHRODINGER"]}})),
        dog_person,
        cat_person
    );
    assert_hits_unordered!(filter(json!({"alive": {"notIn": ["YES", "SCHRODINGER"]}})), dead_person);
    assert_hits_unordered!(filter(json!({"alive": {"notIn": ["YES", "SCHRODINGER"]}})), dead_person);
    assert_hits_unordered!(filter(json!({"favoritePet": {"isNull": true}})), dead_person);
    assert_hits_unordered!(
        filter(json!({"favoritePet": {"isNull": false}})),
        cat_person,
        dog_person
    );
    assert_hits_unordered!(
        filter(json!({"pets": {"includes": {"eq": "HAMSTER"}}})),
        cat_person,
        dog_person
    );

    // Text search
    assert_hits_unordered!(search_text("hamstr"), cat_person, dog_person);
    assert_hits_unordered!(search_text("cat"), cat_person);
    assert_hits_unordered!(search_text("shroidnger"), cat_person);
}

#[cfg(not(feature = "dynamodb"))] // GB-3636
#[test]
fn search_regex() {
    use backend::project::GraphType;

    let mut env = Environment::init();
    env.grafbase_init(GraphType::Single);
    env.write_schema(SEARCH_SCHEMA);
    env.grafbase_dev();
    let client = env.create_client().with_api_key();
    client.poll_endpoint(30, 300);

    let create = |variables: Value| {
        let response = client.gql::<Value>(SEARCH_CREATE_OPTIONAL).variables(variables).send();
        dot_get!(response, "data.fieldsCreate.fields.id", String)
    };
    let search = |variables: Value| {
        let response = client.gql::<Value>(SEARCH_SEARCH_OPTIONAL).variables(variables).send();
        dot_get!(response, "data.fieldsSearch", Collection<Value>)
    };
    let filter = |filter: Value| search(json!({"first": 10, "filter": filter}));

    let dog = create(json!({
        "url": "https://bestfriends.com/",
        "email": "contact@bestfriends.com",
        "phone": "+33612121212",
        "text": "Dogs are the best! Who doesn't love them???",
    }));
    let cat = create(json!({
        "url": "https://cats-world-domination.com/",
        "email": "overlord@cats-world-domination.com",
        "phone": "+33700000000",
        "text": "Cats dominate Youtube today, the World tomorrow.",
    }));

    // URL
    assert_hits_unordered!(filter(json!({"url": {"regex": ".*best.*"}})), dog);
    assert_hits_unordered!(filter(json!({"url": {"regex": ".*world.*"}})), cat);
    assert_hits_unordered!(filter(json!({"url": {"regex": "https.*"}})), cat, dog);

    // Email
    assert_hits_unordered!(filter(json!({"email": {"regex": "contact.*"}})), dog);
    assert_hits_unordered!(filter(json!({"email": {"regex": ".*overlord.*"}})), cat);
    assert_hits_unordered!(filter(json!({"email": {"regex": ".*@.*\\.com"}})), cat, dog);

    // Phone
    assert_hits_unordered!(filter(json!({"phone": {"regex": "\\+336[12]*"}})), dog);
    assert_hits_unordered!(filter(json!({"phone": {"regex": "\\+3370{8}"}})), cat);
    assert_hits_unordered!(filter(json!({"phone": {"regex": "\\+\\d+"}})), cat, dog);

    // Text
    assert_hits_unordered!(filter(json!({"text": {"regex": ".*Dogs.*"}})), dog);
    assert_hits_unordered!(filter(json!({"text": {"regex": "Cats\\s\\w.*"}})), cat);
    assert_hits_unordered!(filter(json!({"text": {"regex": ".*"}})), cat, dog);
}

#[cfg(not(feature = "dynamodb"))] // GB-3636
#[rstest]
#[case("fields", SEARCH_CREATE_OPTIONAL, SEARCH_SEARCH_OPTIONAL)]
#[case("requiredFields", SEARCH_CREATE_REQUIRED, SEARCH_SEARCH_REQUIRED)]
#[case("listFields", SEARCH_CREATE_LIST, SEARCH_SEARCH_LIST)]
fn basic_search(#[case] name: &str, #[case] create_query: &str, #[case] search_query: &str) {
    use backend::project::GraphType;

    let mut env = Environment::init();
    env.grafbase_init(GraphType::Single);
    env.write_schema(SEARCH_SCHEMA);
    env.grafbase_dev();
    let client = env.create_client().with_api_key();
    client.poll_endpoint(30, 300);

    let create = |variables: Value| -> String {
        let response = client.gql::<Value>(create_query).variables(variables).send();
        dot_get!(response, &format!("data.{name}Create.{name}.id"))
    };
    let search = |query: &str, fields: Option<Vec<&str>>, filter: Value| -> Collection<Value> {
        let query = if query.is_empty() {
            Value::Null
        } else {
            Value::String(query.to_string())
        };
        let response = client
            .gql::<Value>(search_query)
            .variables(json!({"query": query, "first": 10, "filter": filter, "fields": fields}))
            .send();
        dot_get!(response, &format!("data.{name}Search"))
    };
    let search_text = |query: &str| search(query, None, Value::Null);
    let filter = |field: &str, condition: Value| {
        let filter = if name == "listFields" {
            json!({ field: {"includes": condition }})
        } else {
            json!({ field: condition })
        };
        search("", None, filter)
    };

    let (dog_fields, cat_fields) = {
        let mut dog_fields = json!({
            "ip": "127.0.0.1",
            "timestamp": 1_451_653_820_000_u64,
            "url": "https://bestfriends.com/",
            "email": "contact@bestfriends.com",
            "phone": "+33612121212",
            "date": "2007-12-03",
            "datetime": "2016-01-01T13:10:20.000Z",
            "text": "Dogs are the best! Who doesn't love them???",
            "int": 8901,
            "float": 23.192,
            "bool": true
        });
        let mut cat_fields = json!({
            "ip": "127.0.0.56",
            "timestamp": 1_641_546_920_000_u64,
            "url": "https://cats-world-domination.com/",
            "email": "overlord@cats-world-domination.com",
            "phone": "+33700000000",
            "date": "2022-01-07",
            "datetime": "2022-01-07T09:15:20.000Z",
            "text": "Cats dominate Youtube today, the World tomorrow.",
            "int": -238,
            "float": 10.27,
            "bool": false
        });
        if name == "listFields" {
            for (_, value) in &mut *dog_fields.as_object_mut().unwrap() {
                *value = serde_json::Value::Array(vec![value.clone()]);
            }
            for (_, value) in &mut *cat_fields.as_object_mut().unwrap() {
                *value = serde_json::Value::Array(vec![value.clone()]);
            }
        }
        (dog_fields, cat_fields)
    };
    let dog_id = create(dog_fields.clone());
    let cat_id = create(cat_fields.clone());
    if name != "requiredFields" {
        // Create an empty record which should never appear
        create(json!({}));
    }

    // ======================
    // == Full-text search ==
    // ======================
    let result = search_text("Dogs");
    assert_eq!(result.edges.len(), 1, "{result:?}");
    // Properly returns all fields
    assert_same_fields!(&result.edges.first().unwrap().node, dog_id, dog_fields);

    let result = search_text("Cats");
    assert_eq!(result.edges.len(), 1, "{result:?}");
    assert_same_fields!(&result.edges.first().unwrap().node, cat_id, cat_fields);

    assert_hits_unordered!(search_text("the"), dog_id, cat_id);
    assert_hits_unordered!(search_text("Dogs best"), dog_id);
    assert_hits_unordered!(search_text("\"Dogs are the best\""), dog_id);

    // Trims whitespace
    assert_hits_unordered!(search_text("the "), dog_id, cat_id);
    assert_hits_unordered!(search_text(" the"), dog_id, cat_id);
    assert_hits_unordered!(search_text("  the  "), dog_id, cat_id);

    // URL / Email
    assert_hits_unordered!(search_text("bestfriends"), dog_id);
    assert_hits_unordered!(search_text("domination"), cat_id);

    // URL
    assert_hits_unordered!(search_text("https"), cat_id, dog_id);

    // Email
    assert_hits_unordered!(search_text("contact"), dog_id);
    assert_hits_unordered!(search_text("overlord"), cat_id);

    // Phone
    assert_hits_unordered!(search_text("+33612121212"), dog_id);
    assert_hits_unordered!(search_text("+33700000000"), cat_id);

    // Fields
    assert_hits_unordered!(search("bestfriends", Some(vec!["url"]), Value::Null), dog_id);
    assert_hits_unordered!(search("cats-world-domination", Some(vec!["url"]), Value::Null), cat_id);
    assert_hits_unordered!(search("bestfriends", Some(vec!["email"]), Value::Null), dog_id);
    assert_hits_unordered!(
        search("cats-world-domination", Some(vec!["email"]), Value::Null),
        cat_id
    );
    // Not finding "Youtube" in email
    let result = search("Youtube", Some(vec!["email"]), Value::Null);
    assert!(result.edges.is_empty(), "{result:?}");

    // =================
    // == String-like ==
    // =================
    assert_hits_unordered!(
        filter("text", json!({"eq": "Dogs are the best! Who doesn't love them???"})),
        dog_id
    );
    assert_hits_unordered!(
        filter(
            "text",
            json!({"eq": "Cats dominate Youtube today, the World tomorrow."})
        ),
        cat_id
    );
    assert_hits_unordered!(filter("text", json!({"gte": "D"})), dog_id);
    assert_hits_unordered!(filter("text", json!({"lt": "D"})), cat_id);

    assert_hits_unordered!(filter("url", json!({"eq": "https://bestfriends.com/"})), dog_id);
    assert_hits_unordered!(
        filter("url", json!({"eq": "https://cats-world-domination.com/"})),
        cat_id
    );

    assert_hits_unordered!(filter("email", json!({"eq": "contact@bestfriends.com"})), dog_id);
    assert_hits_unordered!(
        filter("email", json!({"eq": "overlord@cats-world-domination.com"})),
        cat_id
    );
    assert_hits_unordered!(filter("email", json!({"gte": "m", "lte": "q"})), cat_id);
    assert_hits_unordered!(filter("email", json!({"gte": "a", "lte": "z"})), dog_id, cat_id);
    assert_hits_unordered!(filter("email", json!({"gte": "c", "lte": "e"})), dog_id);
    assert_hits_unordered!(
        filter("email", json!({"in": ["overlord@cats-world-domination.com"]})),
        cat_id
    );
    assert_hits_unordered!(
        filter("email", json!({"notIn": ["overlord@cats-world-domination.com"]})),
        dog_id
    );

    // ==========
    // == Date ==
    // ==========
    assert_hits_unordered!(filter("date", json!({"eq": "2022-01-07"})), cat_id);
    assert_hits_unordered!(filter("date", json!({"eq": "2007-12-03"})), dog_id);
    assert_hits_unordered!(
        filter("date", json!({"gte": "2020-01-01", "lte": "2030-01-01"})),
        cat_id
    );
    assert_hits_unordered!(
        filter("date", json!({"gte": "2000-01-01", "lte": "2010-01-01"})),
        dog_id
    );
    assert_hits_unordered!(
        filter("date", json!({"gte": "2000-01-01", "lte": "2030-01-01"})),
        dog_id,
        cat_id
    );

    assert_hits_unordered!(filter("datetime", json!({"eq": "2016-01-01T13:10:20.000Z"})), dog_id);
    assert_hits_unordered!(filter("datetime", json!({"eq": "2022-01-07T09:15:20.000Z"})), cat_id);
    assert_hits_unordered!(
        filter(
            "datetime",
            json!({"gte": "2010-01-01T00:00:00.000Z", "lte": "2020-01-01T00:00:00.000Z"})
        ),
        dog_id
    );
    assert_hits_unordered!(
        filter(
            "datetime",
            json!({"gte": "2020-01-01T00:00:00.000Z", "lte": "2030-01-01T00:00:00.000Z"})
        ),
        cat_id
    );
    assert_hits_unordered!(
        filter(
            "datetime",
            json!({"gte": "2000-01-01T00:00:00.000Z", "lte": "2030-01-01T00:00:00.000Z"})
        ),
        dog_id,
        cat_id
    );

    assert_hits_unordered!(filter("timestamp", json!({"eq": 1_451_653_820_000_u64})), dog_id);
    assert_hits_unordered!(filter("timestamp", json!({"eq": 1_641_546_920_000_u64})), cat_id);
    assert_hits_unordered!(
        filter(
            "timestamp",
            json!({"gte": 1_400_000_000_000_u64, "lte": 1_500_000_000_000_u64})
        ),
        dog_id
    );
    assert_hits_unordered!(
        filter(
            "timestamp",
            json!({"gte": 1_500_000_000_000_u64, "lte": 1_700_000_000_000_u64})
        ),
        cat_id
    );
    assert_hits_unordered!(
        filter(
            "timestamp",
            json!({"gte": 1_400_000_000_000_u64, "lte": 1_700_000_000_000_u64})
        ),
        dog_id,
        cat_id
    );

    // =============
    // == Numeric ==
    // =============
    assert_hits_unordered!(filter("int", json!({"gte": 8900, "lte": 9000})), dog_id);
    assert_hits_unordered!(filter("int", json!({"gte": -1000, "lte": 9000})), dog_id, cat_id);
    assert_hits_unordered!(filter("int", json!({"gte": -1000, "lte": 0})), cat_id);
    assert_hits_unordered!(filter("int", json!({"eq": 8901})), dog_id);
    assert_hits_unordered!(filter("int", json!({"eq": -238})), cat_id);

    assert_hits_unordered!(filter("float", json!({"gte": 20, "lte": 30})), dog_id);
    assert_hits_unordered!(filter("float", json!({"gte": 10, "lte": 30})), dog_id, cat_id);
    assert_hits_unordered!(filter("float", json!({"gte": 0, "lte": 12})), cat_id);
    assert_hits_unordered!(filter("float", json!({"eq": 23.192})), dog_id);
    assert_hits_unordered!(filter("float", json!({"eq": 10.27})), cat_id);

    // bool
    assert_hits_unordered!(filter("bool", json!({"eq": true})), dog_id);
    assert_hits_unordered!(filter("bool", json!({"eq": false})), cat_id);

    // ip
    assert_hits_unordered!(filter("ip", json!({"eq": "::ffff:7f00:1"})), dog_id);
    assert_hits_unordered!(filter("ip", json!({"eq": "::ffff:7f00:38"})), cat_id);
    assert_hits_unordered!(
        filter("ip", json!({"gte": "::ffff:7f00:1", "lte": "::ffff:7f00:38"})),
        dog_id,
        cat_id
    );

    // ======================
    // == IsNull / IsEmpty ==
    // ======================
    if name == "listFields" {
        assert_hits_unordered!(search("", None, json!({"int": {"isEmpty": false}})), cat_id, dog_id);
        assert_eq!(search("", None, json!({"int": {"isEmpty": true}})).edges.len(), 1);
    } else if name != "requiredFields" {
        // isNull is not present on required fields.
        assert_hits_unordered!(search("", None, json!({"int": {"isNull": false}})), cat_id, dog_id);
        assert_eq!(search("", None, json!({"int": {"isNull": true}})).edges.len(), 1);
    }
}

#[cfg(not(feature = "dynamodb"))] // GB-3636
#[test]
fn search_created_updated_at() {
    use backend::project::GraphType;

    let mut env = Environment::init();
    env.grafbase_init(GraphType::Single);
    env.write_schema(SEARCH_SCHEMA);
    env.grafbase_dev();
    let client = env.create_client().with_api_key();
    client.poll_endpoint(30, 300);

    let create = |variables: Value| -> String {
        let response = client.gql::<Value>(SEARCH_CREATE_OPTIONAL).variables(variables).send();
        dot_get!(response, "data.fieldsCreate.fields.id")
    };
    let search = |variables: Value| -> Collection<Value> {
        let response = client.gql::<Value>(SEARCH_METADATA_FIELDS).variables(variables).send();
        dot_get!(response, "data.fieldsSearch")
    };

    let id1 = create(json!({"int": 1}));
    // FIXME: quite long but Tantivy doesn't seem really consistent with lower threshold for some
    // durations...
    thread::sleep(Duration::from_millis(1000));
    let id2 = create(json!({"int": 2}));
    let (created_at1, created_at2) = {
        let all = search(json!({"first": 10}));
        let mut id_to_created_at = all
            .edges
            .iter()
            .map(|edge| {
                (
                    dot_get!(edge.node, "id", String),
                    dot_get!(edge.node, "createdAt", String),
                )
            })
            .collect::<HashMap<_, _>>();
        (
            id_to_created_at.remove(&id1).unwrap(),
            id_to_created_at.remove(&id2).unwrap(),
        )
    };
    assert!(created_at1 < created_at2, "{created_at1:?} < {created_at2:?}");

    assert_hits_unordered!(
        search(json!({ "first": 10, "filter": { "createdAt": { "gte": created_at2 } } })),
        id2
    );
    assert_hits_unordered!(
        search(json!({ "first": 10, "filter": { "createdAt": { "lte": created_at1 } } })),
        id1
    );

    assert_hits_unordered!(
        search(json!({ "first": 10, "filter": { "updatedAt": { "gte": created_at2 } } })),
        id2
    );
    assert_hits_unordered!(
        search(json!({ "first": 10, "filter": { "updatedAt": { "lte": created_at1 } } })),
        id1
    );
}

#[cfg(not(feature = "dynamodb"))] // GB-3636
#[test]
fn search_pagination_and_total_hits() {
    use backend::project::GraphType;

    let mut env = Environment::init();
    env.grafbase_init(GraphType::Single);
    env.write_schema(SEARCH_SCHEMA);
    env.grafbase_dev();
    let client = env.create_client().with_api_key();
    client.poll_endpoint(30, 300);

    let create = |variables: Value| -> String {
        let response = client.gql::<Value>(SEARCH_CREATE_OPTIONAL).variables(variables).send();
        dot_get!(response, "data.fieldsCreate.fields.id")
    };
    let search = |variables: Value| -> Collection<Value> {
        let response = client.gql::<Value>(SEARCH_PAGINATION).variables(variables).send();
        dot_get!(response, "data.fieldsSearch")
    };

    for i in 1..=10 {
        create(json!({ "int": i }));
    }
    let all = search(json!({"first": 20}));
    let ids = all
        .edges
        .iter()
        .map(|edge| dot_get!(edge.node, "id", String))
        .collect::<Vec<_>>();
    let first_cursor = &all.edges.first().unwrap().cursor;
    let middle_cursor = &all.edges[4].cursor; // 5
    let last_cursor = &all.edges.last().unwrap().cursor;

    assert_hits!(
        search(json!({"first": 20})),
        ids,
        has_next_page: false,
        has_previous_page: false,
        total_hits: 10
    );
    assert_hits!(
        search(json!({"first": 5})),
        &ids[..5],
        has_next_page: true,
        has_previous_page: false,
        total_hits: 10
    );

    let forward = |first: u64, after: &str| search(json!({"first": first, "after": after}));
    assert_hits!(
        forward(20, first_cursor),
        &ids[1..],
        has_next_page: false,
        has_previous_page: false,
        total_hits: 10
    );
    assert_hits!(
        forward(20, middle_cursor),
        &ids[5..],
        has_next_page: false,
        has_previous_page: true,
        total_hits: 10
    );
    assert_hits!(
        forward(20, last_cursor),
        Vec::<String>::new(),
        has_next_page: false,
        has_previous_page: true,
        total_hits: 10
    );
    assert_hits!(
        forward(3, first_cursor),
        &ids[1..4],
        has_next_page: true,
        has_previous_page: false,
        total_hits: 10
    );
    assert_hits!(
        forward(3, middle_cursor),
        &ids[5..8],
        has_next_page: true,
        has_previous_page: true,
        total_hits: 10
    );

    let backward = |last: u64, before: &str| search(json!({"last": last, "before": before}));
    assert_hits!(
        backward(20, first_cursor),
        Vec::<String>::new(),
        has_next_page: true,
        has_previous_page: false,
        total_hits: 10
    );
    assert_hits!(
        backward(20, middle_cursor),
        &ids[..4],
        has_next_page: true,
        has_previous_page: false,
        total_hits: 10
    );
    assert_hits!(
        backward(20, last_cursor),
        &ids[..9],
        has_next_page: false,
        has_previous_page: false,
        total_hits: 10
    );
    assert_hits!(
        backward(3, last_cursor),
        &ids[6..9],
        has_next_page: false,
        has_previous_page: true,
        total_hits: 10
    );
    assert_hits!(
        backward(3, middle_cursor),
        &ids[1..4],
        has_next_page: true,
        has_previous_page: true,
        total_hits: 10
    );
}
