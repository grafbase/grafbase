#![allow(unused_crate_dependencies, clippy::panic)]

use std::process::Command;

#[test]
fn ensure_upstream_is_up_to_date() {
    // The tests in this crate are powered by the upstream graphql-federation-gateway-audit repo
    //
    // I don't want people to have to manually fetch that and keep it up to date, because that sucks.
    //
    // But I also don't want to do the update inline in the audit_tests - because they'll probably
    // be run in parallel and who can be bothered with cross process locking.
    //
    // So, this is a _single_ test that does the up date process and fails if an update was made.
    //
    // This failure should act as a trigger to re-run the tests with the up to date checkout.

    if !should_update() {
        return;
    }

    if !std::fs::exists("gateway-audit-repo").unwrap() {
        let status = Command::new("git")
            .args([
                "clone",
                "git@github.com:the-guild-org/graphql-federation-gateway-audit.git",
                "gateway-audit-repo",
            ])
            .status()
            .unwrap();

        if !status.success() {
            panic!("Could not clone git@github.com:the-guild-org/graphql-federation-gateway-audit.git - please do it yourself");
        }
    }

    let expected_ref = std::fs::read_to_string("AUDIT_REPO_SHA").unwrap();

    let status = Command::new("git")
        .current_dir("gateway-audit-repo")
        .args(["checkout", &expected_ref])
        .status()
        .unwrap();

    if !status.success() {
        panic!("Could not checkout {expected_ref} of graphql-federation-gateway-audit - please do it yourself");
    }

    let status = Command::new("npm")
        .current_dir("gateway-audit-repo")
        .args(["install"])
        .status()
        .unwrap();

    if !status.success() {
        panic!("Could not pnpmi install in graphql-federation-gateway-audit - please do it yourself");
    }

    panic!("Checkout of graphql-federation-gateway-audit was updated.  Please re-run the audit tests")
}

fn should_update() -> bool {
    if !std::fs::exists("gateway-audit-repo").unwrap() {
        return true;
    }

    let output = Command::new("git")
        .current_dir("gateway-audit-repo")
        .args(["rev-parse", "HEAD"])
        .output()
        .unwrap();

    let head_ref = std::str::from_utf8(&output.stdout).unwrap();

    let expected_ref = std::fs::read_to_string("AUDIT_REPO_SHA").unwrap();

    head_ref.trim() != expected_ref.trim()
}
