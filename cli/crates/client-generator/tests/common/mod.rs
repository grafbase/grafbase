use expect_test::Expect;

#[track_caller]
pub fn expect_generated(graphql_schema: impl AsRef<str>, expected: &Expect) {
    let result = grafbase_client_generator::generate(graphql_schema).unwrap();
    expected.assert_eq(&result);
}
