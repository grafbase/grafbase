#![allow(unused_crate_dependencies)]

use federation_audit_tests::{
    audit_server::{AuditServer, Test, TestSuite},
    cached_tests, CachedTest,
};
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
    move || Ok(())
}
