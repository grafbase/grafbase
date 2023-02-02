#![allow(clippy::too_many_lines)]
mod utils;

use std::collections::HashSet;

use rstest::rstest;
use serde_json::{json, Value};
use utils::consts::{
    SEARCH_CREATE_LIST, SEARCH_CREATE_OPTIONAL, SEARCH_CREATE_REQUIRED, SEARCH_SCHEMA, SEARCH_SEARCH_LIST,
    SEARCH_SEARCH_OPTIONAL, SEARCH_SEARCH_REQUIRED,
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

macro_rules! assert_found {
    ($result: expr, $($expected_id:expr),+) => {{
        let expected = HashSet::from([$($expected_id.clone()),+]);
        let result = $result;
        assert_eq!(
            result
                .into_iter()
                .map(|fields| dot_get!(fields, "id", String))
                .collect::<HashSet<_>>(),
            expected
        );
    }};
}

// Tantivy query syntax
// https://docs.rs/tantivy/latest/tantivy/query/struct.QueryParser.html
#[rstest]
#[case("fields", SEARCH_CREATE_OPTIONAL, SEARCH_SEARCH_OPTIONAL, 4021)]
#[case("requiredFields", SEARCH_CREATE_REQUIRED, SEARCH_SEARCH_REQUIRED, 4022)]
#[case("listFields", SEARCH_CREATE_LIST, SEARCH_SEARCH_LIST, 4023)]
fn search(#[case] name: &str, #[case] create_query: &str, #[case] search_query: &str, #[case] port: u16) {
    let mut env = Environment::init(port);
    env.grafbase_init();
    env.write_schema(SEARCH_SCHEMA);
    env.grafbase_dev();
    let client = env.create_client();
    client.poll_endpoint(30, 300);

    let create = |variables: Value| -> String {
        let response = client.gql::<Value>(
            json!({
                "query": create_query,
                "variables": variables
            })
            .to_string(),
        );
        dot_get!(response, &format!("data.{name}Create.{name}.id"))
    };
    let search = |query: &str| -> Vec<Value> {
        let response = client.gql::<Value>(
            json!({
                "query": search_query,
                "variables": json!({"query": query, "limit": 10})
            })
            .to_string(),
        );
        dot_get!(response, &format!("data.{name}Search"))
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
            "email": "admin@cats-world-domination.com",
            "phone": "+33700000000",
            "date": "2022-01-07",
            "datetime": "2022-01-07T09:15:20.000Z",
            "text": "Cats dominate Youtube today, the World tomorrow.",
            "int": -238,
            "float": 10.27,
            "bool": false
        });
        if name == "listFields" {
            for (_, value) in dog_fields.as_object_mut().unwrap().iter_mut() {
                *value = serde_json::Value::Array(vec![value.clone()]);
            }
            for (_, value) in cat_fields.as_object_mut().unwrap().iter_mut() {
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
    let result = search("Dogs");
    assert_eq!(result.len(), 1, "{result:?}");
    // Properly returns all fields
    assert_same_fields!(result.first().unwrap(), dog_id, dog_fields);

    let result = search("Cats");
    assert_eq!(result.len(), 1, "{result:?}");
    assert_same_fields!(result.first().unwrap(), cat_id, cat_fields);

    assert_found!(search("the"), dog_id, cat_id);
    assert_found!(search("Dogs best them"), dog_id);
    assert_found!(search("\"Dogs are the best\""), dog_id);

    // URL
    assert_found!(search("\"https://bestfriends.com/\""), dog_id);
    assert_found!(search("\"https://cats-world-domination.com/\""), cat_id);

    // Email
    assert_found!(search("contact@bestfriends.com"), dog_id);
    assert_found!(search("admin@cats-world-domination.com"), cat_id);

    // Phone
    assert_found!(search("\"+33612121212\""), dog_id);
    assert_found!(search("\"+33700000000\""), cat_id);

    // Across default fields
    assert_found!(search("Dogs \"+33612121212\" contact@bestfriends.com"), dog_id);
    assert_found!(search("Cats \"+33700000000\" admin@cats-world-domination.com"), cat_id);

    assert_found!(search("email:[a TO b]"), cat_id);
    assert_found!(search("email:[a TO e]"), dog_id, cat_id);
    assert_found!(search("email:[c TO e]"), dog_id);

    // ==========
    // == Date ==
    // ==========
    assert_found!(search("date:\"2022-01-07T00:00:00.000Z\""), cat_id);
    assert_found!(search("date:\"2007-12-03T00:00:00.000Z\""), dog_id);
    assert_found!(
        search("date:[2020-01-01T00:00:00.000Z TO 2030-01-01T00:00:00.000Z]"),
        cat_id
    );
    assert_found!(
        search("date:[2000-01-01T00:00:00.000Z TO 2010-01-01T00:00:00.000Z]"),
        dog_id
    );
    assert_found!(
        search("date:[2000-01-01T00:00:00.000Z TO 2030-01-01T00:00:00.000Z]"),
        dog_id,
        cat_id
    );

    assert_found!(search("datetime:\"2016-01-01T13:10:20.000Z\""), dog_id);
    assert_found!(search("datetime:\"2022-01-07T09:15:20.000Z\""), cat_id);
    assert_found!(
        search("datetime:[2010-01-01T00:00:00.000Z TO 2020-01-01T00:00:00.000Z]"),
        dog_id
    );
    assert_found!(
        search("datetime:[2020-01-01T00:00:00.000Z TO 2030-01-01T00:00:00.000Z]"),
        cat_id
    );
    assert_found!(
        search("datetime:[2000-01-01T00:00:00.000Z TO 2030-01-01T00:00:00.000Z]"),
        dog_id,
        cat_id
    );

    assert_found!(search("timestamp:\"2016-01-01T13:10:20.000Z\""), dog_id);
    assert_found!(search("timestamp:\"2022-01-07T09:15:20.000Z\""), cat_id);
    assert_found!(
        search("timestamp:[2010-01-01T00:00:00.000Z TO 2020-01-01T00:00:00.000Z]"),
        dog_id
    );
    assert_found!(
        search("timestamp:[2020-01-01T00:00:00.000Z TO 2030-01-01T00:00:00.000Z]"),
        cat_id
    );
    assert_found!(
        search("timestamp:[2000-01-01T00:00:00.000Z TO 2030-01-01T00:00:00.000Z]"),
        dog_id,
        cat_id
    );

    // =============
    // == Numeric ==
    // =============
    assert_found!(search("int:[8900 TO 9000]"), dog_id);
    assert_found!(search("int:[-1000 TO 9000]"), dog_id, cat_id);
    assert_found!(search("int:[-1000 TO 0]"), cat_id);
    assert_found!(search("int:8901"), dog_id);
    assert_found!(search("int:-238"), cat_id);

    assert_found!(search("float:[20 TO 30]"), dog_id);
    assert_found!(search("float:[10 TO 30]"), dog_id, cat_id);
    assert_found!(search("float:[0 TO 12]"), cat_id);
    assert_found!(search("float:23.192"), dog_id);
    assert_found!(search("float:10.27"), cat_id);

    // bool
    assert_found!(search("bool:true"), dog_id);
    assert_found!(search("bool:false"), cat_id);

    // ip
    assert_found!(search("ip:\"::ffff:7f00:1\""), dog_id);
    assert_found!(search("ip:\"::ffff:7f00:38\""), cat_id);
    assert_found!(search("ip:[::ffff:7f00:1 TO ::ffff:7f00:38]"), dog_id, cat_id);
}
