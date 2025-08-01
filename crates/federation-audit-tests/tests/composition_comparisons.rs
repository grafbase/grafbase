use federation_audit_tests::{audit_server::AuditServer, cached_tests};
use libtest_mimic::{Arguments, Failed, Trial};

fn main() {
    let args = Arguments::from_args();

    let cached_tests = cached_tests();
    let mut suites = cached_tests.iter().map(|test| test.suite()).collect::<Vec<_>>();

    suites.sort();
    suites.dedup();

    let tests = suites
        .into_iter()
        .map(|suite| Trial::test(suite, runner_for(suite.to_string())).with_ignored_flag(true))
        .collect();

    // Run all tests and exit the application appropriatly.
    libtest_mimic::run(&args, tests).exit()
}

fn runner_for(suite: String) -> impl FnOnce() -> Result<(), Failed> + Send + 'static {
    move || {
        let audit_server = AuditServer::new_from_env();
        let suite = audit_server.lookup_suite(suite);
        let expected_supergraph_sdl = suite.supergraph_sdl();

        let mut subgraphs = graphql_composition::Subgraphs::default();
        for subgraph in suite.subgraphs() {
            subgraphs
                .ingest_str(&subgraph.sdl, &subgraph.name, Some(&subgraph.url))
                .unwrap()
        }

        let graph = graphql_composition::compose(subgraphs).into_result().unwrap();
        let output = graphql_composition::render_federated_sdl(&graph).unwrap();

        let output = prettify_sdl(&output);
        let expected = prettify_sdl(&expected_supergraph_sdl);

        assert_eq!(output, expected, "{}", diff(&output, &expected));

        Ok(())
    }
}

fn diff(grafbase: &str, apollo: &str) -> String {
    similar_asserts::SimpleDiff::from_str(grafbase, apollo, "grafbase", "apollo").to_string()
}

// Passes SDL through cynic parser to unify the formatting
fn prettify_sdl(input: &str) -> String {
    cynic_parser::parse_type_system_document(input)
        .unwrap()
        .pretty_printer()
        .sorted()
        .to_string()
}
