#![allow(unused_crate_dependencies)]
mod utils;

use backend::project::GraphType;
use regex::Regex;
use serde_json::{json, Value};
use utils::client::Client;
use utils::consts::{SCALARS_CREATE_OPTIONAL, SCALARS_CREATE_REQUIRED, SCALARS_SCHEMA};
use utils::environment::Environment;

impl Client {
    fn create_opt(&self, variables: Value) -> Value {
        self.gql::<Value>(SCALARS_CREATE_OPTIONAL).variables(variables).send()
    }

    fn create_req(&self, variables: Value) -> Value {
        self.gql::<Value>(SCALARS_CREATE_REQUIRED).variables(variables).send()
    }
}

struct TestCase {
    ty: &'static str,
    input: Value,
    expected: Result<Value, Regex>,
}

impl TestCase {
    fn run_with(self, client: &Client) {
        let TestCase { ty, input, expected } = self;
        let response = client.create_opt(json!({ ty: input }));
        match expected {
            Ok(expected) => {
                let result: Value = dot_get!(response, &format!("data.scalarsCreate.scalars.{}", &ty));
                assert_eq!(result, expected, "{ty}: expected {expected:#?} but got {result:#?}",);
            }
            Err(regex) => {
                // Clippy doesn't like the format call within expect but the suggest alternative of
                // explicitly panicking within a `unwrap_or_else` isn't better.
                #[allow(clippy::expect_fun_call)]
                let result = dot_get_opt!(response, "errors.0.message", String)
                    .expect(&format!("No errors for '{ty}' with: {input:#?}"));
                assert!(regex.is_match(&result), "'{result}' didn't match the pattern '{regex}'");
            }
        }
    }
}

fn error_matching(pattern: &str) -> Result<Value, Regex> {
    Err(Regex::new(pattern).unwrap())
}

// There's no point in splitting test cases
#[allow(clippy::too_many_lines)]
#[test]
#[ignore]
fn scalars() {
    let mut env = Environment::init();
    env.grafbase_init(GraphType::Single);
    env.write_schema(SCALARS_SCHEMA);
    env.grafbase_dev();
    let client = env.create_client().with_api_key();
    client.poll_endpoint(30, 300);

    let variables = json!({
        "ip": "127.0.0.1",
        "timestamp": 1_661_971_087_977_u64,
        "url": "https://example.com/",
        "email": "hello@grafbase.com",
        "json": json!({
            "a": "hello",
            "b": 2,
            "c": json!({
                "a": 11,
                "b": 22
            })
        }),
        "phone": "+33612121212",
        "date": "2007-12-03",
        "datetime": "2016-01-01T13:10:20.000Z"
    });
    // Ensure everything works with correctly formatted inputs whether optional or required.
    for (prefix, response) in [
        ("data.scalarsCreate.scalars", client.create_opt(variables.clone())),
        (
            "data.requiredScalarsCreate.requiredScalars",
            client.create_req(variables.clone()),
        ),
    ] {
        assert_eq!(
            dot_get!(response, &format!("{prefix}.ip"), String),
            dot_get!(variables, "ip", String)
        );
        assert_eq!(
            dot_get!(response, &format!("{prefix}.timestamp"), i64),
            dot_get!(variables, "timestamp", i64)
        );
        assert_eq!(
            dot_get!(response, &format!("{prefix}.url"), String),
            dot_get!(variables, "url", String)
        );
        assert_eq!(
            dot_get!(response, &format!("{prefix}.email"), String),
            dot_get!(variables, "email", String)
        );
        assert_eq!(
            dot_get!(response, &format!("{prefix}.json"), Value),
            dot_get!(variables, "json", Value)
        );
        assert_eq!(
            dot_get!(response, &format!("{prefix}.phone"), String),
            dot_get!(variables, "phone", String)
        );
        assert_eq!(
            dot_get!(response, &format!("{prefix}.date"), String),
            dot_get!(variables, "date", String)
        );
        assert_eq!(
            dot_get!(response, &format!("{prefix}.datetime"), String),
            dot_get!(variables, "datetime", String)
        );
    }

    for test_case in [
        TestCase {
            ty: "datetime",
            input: json!("2016-01-01T13:10:20Z"),
            expected: Ok(json!("2016-01-01T13:10:20.000Z")),
        },
        // Verify conversion are properly applied or that some format are valid.
        TestCase {
            ty: "datetime",
            input: json!("2016-01-01T13:10:20+02:00"),
            expected: Ok(json!("2016-01-01T11:10:20.000Z")),
        },
        TestCase {
            ty: "datetime",
            input: json!("2016-01-01T13:10:20+02:00"),
            expected: Ok(json!("2016-01-01T11:10:20.000Z")),
        },
        TestCase {
            ty: "url",
            input: json!("file://test.com"),
            expected: Ok(json!("file://test.com/")),
        },
        // Stupid URL examples but currently they work...
        TestCase {
            ty: "url",
            input: json!("file://test"),
            expected: Ok(json!("file://test/")),
        },
        TestCase {
            ty: "url",
            input: json!("hello://world"),
            expected: Ok(json!("hello://world")),
        },
        TestCase {
            ty: "date",
            input: json!("01-01-01"),
            expected: Ok(json!("01-01-01")),
        },
        // Ensure we have a validation error with an appropriate message
        TestCase {
            ty: "ip",
            input: json!("-1"),
            expected: error_matching("IP address"),
        },
        TestCase {
            ty: "ip",
            input: json!("-1.0.0.0"),
            expected: error_matching("IP address"),
        },
        TestCase {
            ty: "url",
            input: json!("test.com"),
            expected: error_matching("URL without a base"),
        },
    ] {
        test_case.run_with(&client);
    }

    for invalid_email in [
        "Abc.example.com",
        "A@b@c@example.com",
        "a\"b(c)d,e:f;g<h>i[j\\k]l@example.com",
        "just\"not\"right@example.com",
        "this is\"not\\allowed@example.com",
        "this\\ still\\\"notallowed@example.com",
    ] {
        TestCase {
            ty: "email",
            input: json!(invalid_email),
            expected: error_matching("invalid email address"),
        }
        .run_with(&client);
    }

    for invalid_phone in ["0", "-33612121212", "number"] {
        TestCase {
            ty: "phone",
            input: json!(invalid_phone),
            expected: error_matching("Phone"),
        }
        .run_with(&client);
    }

    for invalid_date in [
        "0000001-01-01",
        "20x0-01-01",
        "2001-00-01",
        "2001-13-01",
        "2001-x1-01",
        "2002-01-00",
        "2002-02-30",
        "2002-01-x1",
    ] {
        TestCase {
            ty: "date",
            input: json!(invalid_date),
            expected: error_matching("Date"),
        }
        .run_with(&client);
    }

    for invalid_dateime in [
        "0000001-01-01T00:00:00Z",
        "2001-01-01",
        "2001-01-01T00:00",
        "2001-01-01T00:00Y",
        "2001-13-01T00:00Z",
        "2001-01-32T00:00Z",
        "2001-01-01T25:00Z",
        "2001-01-01T00:99Z",
        "2001-01-01T00:00+0200Z",
        "2001-01-01T00:00+0200",
    ] {
        TestCase {
            ty: "datetime",
            input: json!(invalid_dateime),
            expected: error_matching("DateTime"),
        }
        .run_with(&client);
    }
}
