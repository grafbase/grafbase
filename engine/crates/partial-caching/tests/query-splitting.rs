#![allow(unused_crate_dependencies)]

const SCHEMA: &str = r#"
    type Query {
        user: User @resolver(name: "whatever")
    }

    type User {
        name: String @cache(maxAge: 140)
        email: String @cache(maxAge: 130)
        someConstant: String @cache(maxAge: 120)
        nested: String @resolver(name: "whatever")
    }

    type Nested {
        someThing: String @cache(maxAge: 140)
    }
"#;

#[test]
fn test_basic_split() {
    let registry = build_registry(SCHEMA);

    const QUERY: &str = "query { user { name email someConstant } }";

    let plan = partial_caching::build_plan(QUERY, None, &registry).unwrap().unwrap();

    let result = plan
        .cache_partitions
        .into_iter()
        .map(|(_, query)| query.as_display(&plan.document).to_string())
        .collect::<Vec<_>>();

    assert_eq!(result.len(), 3, "{result:?}");

    insta::assert_snapshot!(result[0], @r###"
    query {
      user {
        name
      }
    }
    "###);

    insta::assert_snapshot!(result[1], @r###"
    query {
      user {
        email
      }
    }
    "###);

    insta::assert_snapshot!(result[2], @r###"
    query {
      user {
        someConstant
      }
    }
    "###);
}

#[test]
fn test_split_with_absurd_fragments() {
    let registry = build_registry(SCHEMA);

    const QUERY: &str = r#"
        query {
            ...UserFragment
            user {
                name
                email
                ...ConstantFragment
            }
        }

        fragment UserFragment on Query {
            user {
                name
                ...NameAndEmailFragment
                ...EmailFragment
                ...ConstantFragment
            }
        }

        fragment NameAndEmailFragment on User {
            name
            ...EmailFragment
        }

        fragment EmailFragment on User {
            email
        }

        fragment ConstantFragment on User {
            someConstant
        }
    "#;

    let plan = partial_caching::build_plan(QUERY, None, &registry).unwrap().unwrap();

    let result = plan
        .cache_partitions
        .into_iter()
        .map(|(_, query)| query.as_display(&plan.document).to_string())
        .collect::<Vec<_>>();

    assert_eq!(result.len(), 3, "{result:?}");

    // This one should have all the references to user
    insta::assert_snapshot!(result[0], @r###"
    query {
      ... UserFragment
      user {
        name
      }
    }

    fragment UserFragment on Query {
      user {
        name
        ... NameAndEmailFragment
      }
    }

    fragment NameAndEmailFragment on User {
      name
    }
    "###);

    // This one should have all the references to email
    insta::assert_snapshot!(result[1], @r###"
    query {
      ... UserFragment
      user {
        email
      }
    }

    fragment EmailFragment on User {
      email
    }

    fragment NameAndEmailFragment on User {
      ... EmailFragment
    }

    fragment UserFragment on Query {
      user {
        ... NameAndEmailFragment
        ... EmailFragment
      }
    }
    "###);

    // This one should have all the references to someConstant
    insta::assert_snapshot!(result[2], @r###"
    query {
      ... UserFragment
      user {
        ... ConstantFragment
      }
    }

    fragment ConstantFragment on User {
      someConstant
    }

    fragment UserFragment on Query {
      user {
        ... ConstantFragment
      }
    }
    "###);
}

#[test]
fn test_arguments_and_directives_preserved() {
    let registry = build_registry(
        r#"
        type Query {
            user: String @resolver(name: "whatever")
        }
        type Mutation {
            createUser: String @resolver(name: "whatever")
        }
        "#,
    );

    const QUERY: &str = r#"
    query @whatever {
        user(id: 1) @whatever {
            ... @defer(if: $hello) {
                hello
            }
            ...Whatever @whatever
            ... on User @defer(if: $hello) {
                hello
            }
        }
    }

    fragment Whatever on User @whatever {
        hello
    }
    "#;

    let plan = partial_caching::build_plan(QUERY, None, &registry).unwrap().unwrap();

    assert!(plan.cache_partitions.is_empty());

    let result = plan.nocache_partition.as_display(&plan.document).to_string();

    insta::assert_snapshot!(result)
}

#[test]
fn test_mutation() {
    let registry = build_registry(
        r#"
        type Query {
            user: String @resolver(name: "whatever")
        }
        type Mutation {
            createUser: String @resolver(name: "whatever")
        }
        "#,
    );

    const MUTATION: &str = "mutation { createUser }";

    assert!(partial_caching::build_plan(MUTATION, None, &registry)
        .unwrap()
        .is_none());
}

#[test]
fn test_subscription() {
    let registry = build_registry(
        r#"
        type Query {
            user: String @resolver(name: "whatever")
        }
        type Mutation {
            createUser: String @resolver(name: "whatever")
        }
        "#,
    );

    const SUBSCRIPTION: &str = "subscription { createUser }";

    assert!(partial_caching::build_plan(SUBSCRIPTION, None, &registry)
        .unwrap()
        .is_none());
}

fn build_registry(schema: &str) -> registry_for_cache::PartialCacheRegistry {
    registry_upgrade::convert_v1_to_partial_cache_registry(parser_sdl::parse_registry(schema).unwrap()).unwrap()
}
