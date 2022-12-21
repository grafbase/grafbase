mod utils;

use serde_json::{json, Value};
use utils::consts::{SCALARS_MUTATION, SCALARS_QUERY, SCALARS_SCHEMA};
use utils::environment::Environment;

#[test]
fn scalars() {
    let mut env = Environment::init(4011);

    env.grafbase_init();

    env.write_schema(SCALARS_SCHEMA);

    env.grafbase_dev();

    let client = env.create_client();

    client.poll_endpoint(30, 300);

    client.gql::<Value>(json!({ "query": SCALARS_MUTATION }).to_string());

    let response = client.gql::<Value>(json!({ "query": SCALARS_QUERY }).to_string());

    let first_entity: Value = dot_get!(response, "data.scalarsCollection.edges.0.node");

    let first_entity_id: String = dot_get!(first_entity, "id");

    let date: String = dot_get!(first_entity, "date");
    let datetime: String = dot_get!(first_entity, "datetime");

    assert!(first_entity_id.starts_with("scalars_"));
    assert_eq!(datetime, "2016-01-01T13:10:20Z");
    assert_eq!(date, "2007-12-03");
}
