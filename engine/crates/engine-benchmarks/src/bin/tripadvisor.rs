#![allow(unused_crate_dependencies)]

use integration_tests::{EngineBuilder, ResponseExt};
use serde_json::{json, Value};

fn main() {
    let result = serde_json::from_str(
        &std::fs::read_to_string("/Users/graeme/src/grafbase/tripadvisor-repro/parse-result.json").unwrap(),
    )
    .unwrap();

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    runtime.block_on(async move {
        let engine = EngineBuilder::new("").with_forced_parse_result(result).build().await;

        let result = engine
            .execute(QUERY)
            .variables(json!(
                {
                    "listId": 67,
                    "countryId": 1
                }
            ))
            .await;

        let result = result.assert_success();

        eprintln!("{}", result.into_data::<Value>())
    });
}

const QUERY: &str = "
query ShelfItems($listId: Float!, $countryId: Float!) {
    db {
      listItems(listId: $listId, countryId: $countryId, limit: 9) {
        id
        title
        image
        subjectId
        subjectReferenceId
        destination {
          id
          slug
        }
        departurePort {
          id
          name
        }
        port {
          id
          slug
        }
        ship {
          id
          slug
        }
        cruiseLine {
          id
          slug
        }
      }
    }
  }
";
