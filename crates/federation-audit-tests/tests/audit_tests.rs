use federation_audit_tests::{
    CachedTest, Response,
    audit_server::{AuditServer, ExpectedResponse, Test},
    cached_tests,
};
use integration_tests::gateway::GatewayBuilder;
use libtest_mimic::{Arguments, Failed, Trial};

fn main() {
    let args = Arguments::from_args();

    let tests = cached_tests()
        .into_iter()
        .map(|test| Trial::test(test.name(), runner_for(test)))
        .collect();

    // Run all tests and exit the application appropriatly.
    libtest_mimic::run(&args, tests).exit();
}

fn runner_for(test: CachedTest) -> impl FnOnce() -> Result<(), Failed> + Send + 'static {
    move || {
        // TODO
        let audit_server = AuditServer::new_from_env();
        let (suite, test) = audit_server.lookup_test(test);

        let supergraph_sdl = suite.supergraph_sdl();

        integration_tests::runtime().block_on(run_test(supergraph_sdl, test));

        Ok(())
    }
}

async fn run_test(supergraph_sdl: String, mut test: Test) {
    let server = GatewayBuilder::default()
        .with_federated_sdl(&supergraph_sdl)
        .build()
        .await;

    let response = server.post(test.query).await;

    test.expected.data = floatify_numbers(test.expected.data);

    let response = Response {
        data: floatify_numbers(response.body["data"].clone()),
        errors: &response.errors(),
    };

    if response != test.expected {
        assert_eq!(response, test.expected, "\n\n{}", json_diff(&response, &test.expected));
    }
}

fn json_diff(response: &Response<'_>, expected: &ExpectedResponse) -> String {
    let expected = serde_json::to_string_pretty(expected).unwrap();
    let actual = serde_json::to_string_pretty(response).unwrap();
    similar_asserts::SimpleDiff::from_str(&expected, &actual, "expected", "actual").to_string()
}

/// Converts all the numbers in a Value to float so we can compare them
/// without worrying about comparing integers to floats
fn floatify_numbers(value: serde_json::Value) -> serde_json::Value {
    use serde_json::Value;

    match value {
        Value::Number(number) => Value::Number(serde_json::Number::from_f64(number.as_f64().unwrap()).unwrap()),
        Value::Array(vec) => Value::Array(vec.into_iter().map(floatify_numbers).collect()),
        Value::Object(map) => Value::Object(
            map.into_iter()
                .map(|(key, value)| (key, floatify_numbers(value)))
                .collect(),
        ),
        _ => value,
    }
}
