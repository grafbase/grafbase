use crate::common::expect_ts;
use expect_test::expect;
use grafbase_client_generator::Import;

#[test]
fn import_all() {
    let import = Import::all_as("graphql-request", "gql");

    let expected = expect![[r#"
            import * as gql from 'graphql-request'
        "#]];

    expect_ts(import, &expected);
}

#[test]
fn import_one() {
    let import = Import::items("graphql-request", &["gql"]);

    let expected = expect![[r#"
            import gql from 'graphql-request'
        "#]];

    expect_ts(import, &expected);
}

#[test]
fn import_many() {
    let import = Import::items("graphql-request", &["gql", "GraphQLClient"]);

    let expected = expect![[r#"
            import { gql, GraphQLClient } from 'graphql-request'
        "#]];

    expect_ts(import, &expected);
}
