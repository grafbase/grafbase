#![allow(unused_crate_dependencies, clippy::panic)]

use federation_audit_tests::{audit_server::AuditServer, cached_tests, CachedTest};

#[test]
fn ensure_test_cache_fresh() {
    // The way libtest mimic works means we need to create a list of tests on every startup
    // of the harness. Fetching this from the audit server is a tiny bit slow -
    // like 1.2 seconds
    //
    // Since nextests starts the harness once for every single test this means each test
    // takes 1.2 seconds + the actual time for the test.
    //
    // I don't like this, so I'm writing the list of tests into a json file that the harness
    // can then make use of for maximum speed.
    //
    // _This_ test makes sure that it's up to date.  If it's not up to date it will update it
    // so that you just need to rerun the tests to get a passing run
    // (but on CI it should always just fail)

    let audit_server = AuditServer::new_from_env();

    let tests = audit_server
        .test_suites()
        .into_iter()
        .flat_map(|suite| {
            suite
                .tests()
                .into_iter()
                .enumerate()
                .map(|(i, _)| CachedTest::new(&suite.id, i))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    if tests != cached_tests() {
        if std::env::var("CI") != Ok("true".into()) {
            std::fs::write("tests.json", serde_json::to_vec_pretty(&tests).unwrap()).unwrap();
            panic!("tests.json was not up to date.  it has been updated now, but if you weren't expecting this you should watch out");
        }
        panic!("tests.json is not up to date.  run `cargo test -p federation-audit-tests --test cache_freshness` to update");
    }
}
