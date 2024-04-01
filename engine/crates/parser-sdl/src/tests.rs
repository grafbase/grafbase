use std::collections::{HashMap, HashSet};

use common_types::UdfKind;
use serde_json as _;

macro_rules! assert_validation_error {
    ($schema:expr, $expected_message:literal) => {
        assert_matches!(
            $crate::parse_registry($schema)
                .err()
                .and_then(crate::Error::validation_errors)
                // We don't care whether there are more errors or not.
                // It only matters that we find the expected error.
                .and_then(|errors| errors.into_iter().next()),
            Some(crate::RuleError { message, .. }) => {
                assert_eq!(message, $expected_message);
            }
        )
    };
}

pub(crate) use assert_validation_error;

#[test]
fn should_have_unique_fields() {
    assert_validation_error!(
        r"
        type Product {
            count: Int
            count: Int
        }
        ",
        "Field 'count' cannot be defined multiple times."
    );
}

#[test]
fn should_pick_up_required_resolvers() {
    let variables = HashMap::new();
    const SCHEMA: &str = r#"
        extend type Query {
            user: User! @resolver(name: "user/get-user")
        }

        type User {
            name: String!
            email: String!
            lastSignIn: DateTime
            daysInactive: Int! @resolver(name: "user/days-inactive")
        }

        type Post {
            author: User!
            contents: String!
            computedSummary: String! @resolver(name: "text/summary")
        }

        type Comment {
            author: User!
            post: Post!
            contents: String!
            computedSummary: String! @resolver(name: "text/summary")
        }
    "#;

    let result = super::to_parse_result_with_variables(SCHEMA, &variables).expect("must succeed");

    assert_eq!(
        result.required_udfs,
        HashSet::from([
            (UdfKind::Resolver, "user/days-inactive".to_owned()),
            (UdfKind::Resolver, "user/get-user".to_owned()),
            (UdfKind::Resolver, "text/summary".to_owned())
        ])
    );
}

#[test]
fn should_not_support_search_directive() {
    let simple = super::parse_registry(
        r"
            type Product @model {
                title: String @search
            }
            ",
    );
    insta::assert_debug_snapshot!(simple, @r###"
    Err(
        Validation(
            [
                RuleError {
                    locations: [
                        Pos(2:18),
                    ],
                    message: "The connector-less `@model` directive is no longer supported.",
                },
            ],
        ),
    )
    "###);
}

#[test]
fn test_experimental() {
    let result = super::parse_registry(
        r"
            extend schema @experimental(kv: true)
        ",
    )
    .unwrap();

    assert!(result.enable_kv);

    let result = super::parse_registry(
        r"
            extend schema @experimental(kv: false)
        ",
    )
    .unwrap();

    assert!(!result.enable_kv);
}

#[test]
fn mongodb_join() {
    super::parse_registry(r###"
    extend schema
      @mongodb(
        namespace: false
        name: "MongoDB"
        url: "https://mongodb-api.com/something"
        apiKey: "some-key"
        dataSource: "Cluster0"
        database: "sample_analytics"
      )

    extend type Customer {
      customerContacts: ContactConnection @join(select: "contactCollection(first: 10, filter: { businessid: { eq: $customerid } })")
    }

    type Customer @model(connector: "MongoDB", collection: "customer") {
      customerid: String!
      category: String!
      segment: String!
      status: String!
    }

    type Contact @model(connector: "MongoDB", collection: "contacts") {
      businessid: String!
      parentbusinessid: String!
      businessentitytype: String!
      contactMedium: [ContactMedium!]
    }

    type ContactMedium {
      type: String
      emailid: String
      mobilenumber: String
      address: [Address!]
    }

    type Address {
      address1: String
      address2: String
      city: String
      pincode: String
    }

    "###).unwrap();
}
