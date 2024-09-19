#![allow(unused_crate_dependencies)]

use federation_audit_tests::audit_server::{AuditServer, Test, TestSuite};
use libtest_mimic::{Arguments, Failed, Trial};

fn main() {
    let args = Arguments::from_args();

    let audit_server = AuditServer::new_from_env();

    // TODO: this is going to be slow as hell.  at the very least parallelise it
    // or possibly figure out how to skip it altogether
    let tests = audit_server
        .test_suites()
        .into_iter()
        .flat_map(|suite| {
            suite
                .tests()
                .into_iter()
                .enumerate()
                .map(|(i, test)| Trial::test(format!("{}::{}", suite.id, i), runner_for(suite.clone(), test)))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    // Run all tests and exit the application appropriatly.
    libtest_mimic::run(&args, tests).exit();
}

fn runner_for(suite: TestSuite, test: Test) -> impl FnOnce() -> Result<(), Failed> + Send + 'static {
    move || Ok(())
}
