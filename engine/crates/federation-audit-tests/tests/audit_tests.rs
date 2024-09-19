#![allow(unused_crate_dependencies)]

use federation_audit_tests::{
    audit_server::{AuditServer, ExpectedResponse, Test},
    cached_tests, CachedTest,
};
use integration_tests::federation::TestGatewayBuilder;
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

async fn run_test(supergraph_sdl: String, test: Test) {
    let server = TestGatewayBuilder::default()
        .with_federated_sdl(&supergraph_sdl)
        .build()
        .await;

    let response = server.post(test.query).await;

    similar_asserts::assert_eq!(
        ExpectedResponse {
            data: response.body["data"].clone(),
            errors: !response.errors().is_empty()
        },
        test.expected
    );
}
